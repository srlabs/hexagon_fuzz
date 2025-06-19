#!/usr/bin/env python3

# ---------------------------------------------------------------------
# Be sure to add the python path that points to the LLDB shared library.
#
# # To use this in the embedded python interpreter using "lldb" just
# import it with the full path using the "command script import"
# command
#   (lldb) command script import /path/to/cmdtemplate.py
# ---------------------------------------------------------------------

from __future__ import print_function

import inspect
import lldb
import optparse
import shlex
import sys


def _get_ints(reg_groups):
    sysregs = reg_groups["System Registers"]
    ipendad = sysregs["ipendad"]
    ipend = ipendad & 0x0FFFF
    iad = (ipendad >> 16) & 0x0FFFF

    return ipend, iad


def _print_ints(reg_groups):
    ipend, iad = _get_ints(reg_groups)
    pending = [i for i in range(16) if (1 << i) & ipend] if ipend else None
    disabled = [i for i in range(16) if (1 << i) & iad] if iad else None

    print(
        "IPEND: 0b{:08b} [pending: {}]\n"
        "IAD:   0b{:08b} [auto-disabled: {}]".format(
            ipend, pending, iad, disabled
        )
    )


def _get_vids(reg_groups):
    sysregs = reg_groups["System Registers"]
    VID_MASK = 0x0FF
    VID1_SHIFT = VID3_SHIFT = 16
    VID = sysregs["vid"]
    VID1 = sysregs["vid1"]

    return (
        VID & VID_MASK,  # VID[0]
        (VID >> VID1_SHIFT) & VID_MASK,  # VID[1]
        VID1 & VID_MASK,  # VID[2]
        (VID1 >> VID3_SHIFT) & VID_MASK,  # VID[3]
    )


def _print_vids(reg_groups):
    vids = _get_vids(reg_groups)
    vid_text = [
        "VID{}: 0x{:02x}".format(index, vid) for index, vid in enumerate(vids)
    ]
    print(", ".join(vid_text))


def _to_regs(regs):
    def to_int(name, val):
        try:
            return int(val, 0)
        except TypeError:
            # FIXME: vec regs choke here
            return None

    return dict([(r.name, to_int(r.name, r.value)) for r in regs])


def _get_reg_groups(frame):
    return dict([(regs.name, _to_regs(regs)) for regs in frame.GetRegisters()])


def get_unexpected(sysregs):
    valid_bits = {
        "stid": 0x00FF00FF,
        "imask": 0x0000FFFF,
        "gevb": 0xFFFFFF00,
        "evb": 0xFFFFFF00,
        "modectl": 0x00FF00FF,
        "bestwait": 0x000001FF,
    }
    return [
        reg for reg, mask in valid_bits.items() if (sysregs[reg] & (~mask)) != 0
    ]


def _is_weird_state(sysregs):
    # 'weird' is defined as:
    # (1) having bits set in odd/unexpected places in system registers.
    # (2) having inconsistent values among system registers.
    # (3) PC points to something that lldb can't disassemble?
    # ... ?
    stid = sysregs["stid"]
    valid_bits = {
        "stid": 0x00FF00FF,
        "imask": 0x0000FFFF,
        "gevb": 0xFFFFFF00,
        "evb": 0xFFFFFF00,
        "modectl": 0x00FF00FF,
        "bestwait": 0x000001FF,
    }
    unexpected_bits_set = len(get_unexpected(sysregs)) != 0

    return unexpected_bits_set


def _is_in_handler(sysregs):
    # interrupts
    ssr = sysregs["ssr"]
    if ssr == None:
        return False

    ssr_ex = (ssr & (1 << 17)) != 0

    return not ssr_ex


def _has_interrupt(sysregs):
    # interrupts
    ipendad = sysregs["ipendad"]
    if ipendad == None:
        return False

    ipend = ipendad & 0x0FFFF

    return ipend != 0 and not _is_in_handler(sysregs)


class SysStatCommand(object):
    program = "sysstatus"

    @classmethod
    def register_lldb_command(cls, debugger, module_name):
        parser = cls.create_options()
        cls.__doc__ = parser.format_help()
        # Add any commands contained in this module to LLDB
        command = "command script add -c %s.%s %s" % (
            module_name,
            cls.__name__,
            cls.program,
        )
        debugger.HandleCommand(command)
        print(
            'The "{0}" command has been installed, type "help {0}" or "{0} '
            '--help" for detailed help.'.format(cls.program)
        )

    @classmethod
    def create_options(cls):
        usage = "usage: %prog [options]"
        description = "This command is ... tbd"

        # Pass add_help_option = False, since this keeps the command in line
        #  with lldb commands, and we wire up "help command" to work by
        # providing the long & short help methods below.
        parser = optparse.OptionParser(
            description=description,
            prog=cls.program,
            usage=usage,
            add_help_option=False,
        )

        parser.add_option(
            "-t",
            "--tbd",
            action="store_true",
            dest="tbd",
            help="tbd = True",
            default=True,
        )

        return parser

    def get_short_help(self):
        return "Example command for use in debugging"

    def get_long_help(self):
        return self.help_string

    def __init__(self, debugger, unused):
        self.parser = self.create_options()
        self.help_string = self.parser.format_help()

    def __call__(self, debugger, command, exe_ctx, result):
        # Use the Shell Lexer to properly parse up command options just like a
        # shell would
        command_args = shlex.split(command)

        target = debugger.GetTargetAtIndex(0)

        try:
            (options, args) = self.parser.parse_args(command_args)
        except:
            # if you don't handle exceptions, passing an incorrect argument to
            # the OptionParser will cause LLDB to exit (courtesy of OptParse
            # dealing with argument errors by throwing SystemExit)
            result.SetError("option parsing failed")
            return

        # Always get program state from the lldb.SBExecutionContext passed
        # in as exe_ctx
        frame = exe_ctx.GetFrame()
        if not frame.IsValid():
            result.SetError("invalid frame")
            return

        process = exe_ctx.GetProcess()
        th_count = process.GetNumThreads()
        threads = {
            tid: process.GetThreadAtIndex(tid) for tid in range(th_count)
        }
        frames_by_thread = {
            tid: process.GetThreadAtIndex(tid).GetFrameAtIndex(0)
            for tid in range(th_count)
        }
        function = frame.GetFunction()

        sysregs_by_thread = {
            tid: _get_reg_groups(thread.GetSelectedFrame())
            for tid, thread in threads.items()
        }

        ssr_bits_by_tid = {}
        prio_by_tid = {}
        imask_by_tid = {}
        xa_by_tid = {}
        pc_by_tid = {}
        # mode
        for tid, reg_groups in sysregs_by_thread.items():
            sysregs = reg_groups["System Registers"]
            thread_regs = reg_groups["Thread Registers"]
            ssr = sysregs["ssr"]
            syscfg = sysregs["syscfg"]
            stid = sysregs["stid"]

            if ssr == None or syscfg == None or stid == None:
                prio_by_tid[tid] = None
                imask_by_tid[tid] = None
                continue
            ASID_START = 8
            ASID_MASK = 0b0111_1111
            asid = (ssr >> ASID_START) & ASID_MASK

            ssr_bits = {
                "UM": 16,
                "EX": 17,
                "IE": 18,
                "GM": 19,
                "SS": 30,
                "XE": 31,
            }
            asserted = frozenset(
                [
                    name
                    for name, bit in ssr_bits.items()
                    if (ssr & (1 << bit)) != 0
                ]
            )
            not_asserted = frozenset(
                [
                    name
                    for name, bit in ssr_bits.items()
                    if (ssr & (1 << bit)) == 0
                ]
            )
            CAUSE_START = 0
            CAUSE_MASK = 0x0FF
            cause = (
                ((ssr >> CAUSE_START) & CAUSE_MASK)
                if "EX" in asserted
                else None
            )

            ssr_bits_by_tid[tid] = (asid, asserted, not_asserted, cause)

            XA_START = 24
            XA_MASK = 0b111
            xa_by_tid[tid] = (ssr >> XA_START) & XA_MASK

            syscfg_bits = {
                "PRIO": 14,
                "V2X": 7,
                "TL": 11,
                "KL": 12,
                "G": 4,
                "M": 0,
            }
            syscfg_set = frozenset(
                [
                    name
                    for name, bit in syscfg_bits.items()
                    if (syscfg & (1 << bit)) != 0
                ]
            )
            syscfg_unset = frozenset(
                [
                    name
                    for name, bit in syscfg_bits.items()
                    if (syscfg & (1 << bit)) == 0
                ]
            )

            PRIO_START = 16
            PRIO_MASK = 0b0_1111_1111
            prio = (stid >> PRIO_START) & PRIO_MASK
            prio_by_tid[tid] = prio

            pc_by_tid[tid] = thread_regs["pc"]

            imask = sysregs["imask"]
            imask = imask & 0x0FF
            masked = [i for i in range(16) if (1 << i) & imask]
            int_enabled = "G" in syscfg_set and "IE" in asserted
            imask_by_tid[tid] = (imask, masked, int_enabled)

        reg_groups = _get_reg_groups(frame)
        sysregs = reg_groups["System Registers"]
        if sysregs["modectl"] == None:
            print("unavailable system regs")
            return

        modectl = sysregs["modectl"]
        isdbst = sysregs["isdbst"]

        def to_dwe(thread_num):
            d = bool(isdbst & (1 << thread_num))
            w = bool(modectl & (1 << (thread_num + 16)))
            e = bool(modectl & (1 << thread_num))
            return (int(d), int(w), int(e))

        def decode_dwe(dwe):
            if dwe == (0, 0, 0):
                return "OFF"
            elif dwe == (0, 0, 1):
                return "RUN"
            elif dwe == (0, 1, 1):
                return "WAIT"
            elif dwe == (1, 0, 1):
                return "DEBUG"
            else:
                return "INVALID"

        def priv_mode(ssr_set):
            if "UM" in ssr_set and "GM" not in ssr_set and "EX" not in ssr_set:
                return "User"
            elif "UM" in ssr_set and "GM" in ssr_set and "EX" not in ssr_set:
                return "Guest"
            else:
                return "Monitor"

        thread_dwes = dict([(x, to_dwe(x)) for x in range(th_count)])
        print("modectl:  0x{:08x}".format(modectl))

        BESTWAIT_PRIO_MASK = 0b01_1111_1111
        bestwait_prio = sysregs["bestwait"] & BESTWAIT_PRIO_MASK

        SCHEDCFG_INTNO_MASK = 0b0_1111
        schedcfg = sysregs["schedcfg"]
        schedcfg_intno = schedcfg & SCHEDCFG_INTNO_MASK
        schedcfg_en = (schedcfg >> 8) & 1
        print(
            "bestwait: 0x{:02x} / {} (dec)".format(bestwait_prio, bestwait_prio)
        )
        print(
            "schedcfg: 0x{:08x} - int #{:02x} / {} (dec), EN:{}".format(
                schedcfg,
                schedcfg_intno,
                schedcfg_intno,
                "enabled" if schedcfg_en else "disabled",
            )
        )
        print("syscfg:   0x{:08x}".format(syscfg))

        print(
            "TID Prio Mode  Priv    Cause    Set            Unset\n"
            "--- ---- ----- ------- ----- --------------   --------------"
        )
        for thread_id, dwe in thread_dwes.items():
            thread_mode = decode_dwe(dwe)
            try:
                asid, ssr_set, ssr_unset, cause = ssr_bits_by_tid[thread_id]
            except KeyError:
                asid, ssr_set, ssr_unset, cause = (
                    0xFF,
                    frozenset(),
                    frozenset(),
                    None,
                )
            bits_set = sorted(list(ssr_set))
            bits_unset = sorted(list(ssr_unset))
            prio = prio_by_tid.get(thread_id, "?")

            cause_str = " 0x{:02x}".format(cause) if cause else "  -  "

            print(
                "{:3} {:4} {:5} {:7} {} {:15}  {:15}".format(
                    thread_id,
                    prio,
                    thread_mode,
                    priv_mode(ssr_set),
                    cause_str,
                    ",".join(bits_set),
                    ",".join(bits_unset),
                )
            )
        #           if 'EX' in bits_set:
        #               debugger.HandleCommand('thread select {}'.format(thread_id))
        #               debugger.HandleCommand('register read ssr elr badva0 badva1')
        #               print('\n')
        print(
            "GLB    -       -         -   {:15}  {:15}".format(
                ",".join(syscfg_set), ",".join(syscfg_unset)
            )
        )

        print("\n")

        # ssr:
        #       print('-\n'
        #             'IE: interrupt enable, UM: user mode, GM: guest mode, EX: int/exception accepted\n'
        #             'XE: coproc enable, SS: Single Step')
        # syscfg:
        #       print('V2X: HVX Vec size, TL: TLB lock, KL: kernel lock, G: global int enable, M: MMU enable\n'
        #             'PRIO: scheduling enable\n\n')

        debugger.HandleCommand("thread list")

        print("\n")

        print("TID  Int    IMASK\n" "    Enabled\n" "--- ------- ----------")
        for thread_id, (imask, masked, int_enabled) in imask_by_tid.items():
            print(
                "{:-3} {:7} 0b{:08b}  {}".format(
                    thread_id,
                    str(int_enabled),
                    imask,
                    ", ".join(str(m) for m in masked),
                )
            )

        # interrupts
        ipendad = sysregs["ipendad"]
        ipend = ipendad & 0x0FFFF
        iad = (ipendad >> 16) & 0x0FFFF

        vids = _get_vids(reg_groups)

        _print_ints(reg_groups)
        _print_vids(reg_groups)

        unexpected = get_unexpected(sysregs)
        if unexpected:
            print("regs w/unexpected bits:")
            for reg in unexpected:
                print("\t{:12}: {:08x}".format(reg, sysregs[reg]))


class StepHVXChange(object):
    def __init__(self, thread_plan, dict_):
        self.thread_plan = thread_plan
        frame = thread_plan.GetThread().GetFrameAtIndex(0)
        self.start_hvx_info = self._get_hvx_info()

    def _get_hvx_info(self):
        reg_groups = _get_reg_groups(
            self.thread_plan.GetThread().GetFrameAtIndex(0)
        )
        sysregs = reg_groups["System Registers"]
        XE = (sysregs["ssr"] & (1 << 31)) != 0
        return XE

    def explains_stop(self, event):
        # We are stepping, so if we stop for any other reason, it isn't
        # because of us.
        if (
            self.thread_plan.GetThread().GetStopReason()
            == lldb.eStopReasonTrace
        ):
            return True
        else:
            return False

    def should_stop(self, event):
        if self._get_hvx_info() != self.start_hvx_info:
            self.thread_plan.SetPlanComplete(True)
            return True
        else:
            #           print('continued at 0x{:08x}'.format(this_frame.GetPC()))
            return False

    def should_step(self):
        return True


class StepModeChange(object):
    def __init__(self, thread_plan, dict_):
        self.thread_plan = thread_plan
        frame = thread_plan.GetThread().GetFrameAtIndex(0)
        self.start_mode = self._get_mode()

    def _get_mode(self):
        reg_groups = _get_reg_groups(
            self.thread_plan.GetThread().GetFrameAtIndex(0)
        )
        sysregs = reg_groups["System Registers"]
        return sysregs["modectl"]

    def _is_weird(self):
        reg_groups = _get_reg_groups(
            self.thread_plan.GetThread().GetFrameAtIndex(0)
        )
        sysregs = reg_groups["System Registers"]
        return _is_weird_state(sysregs)

    def explains_stop(self, event):
        # We are stepping, so if we stop for any other reason, it isn't
        # because of us.
        if (
            self.thread_plan.GetThread().GetStopReason()
            == lldb.eStopReasonTrace
        ):
            return True
        else:
            return False

    def should_stop(self, event):
        if self._get_mode() != self.start_mode or self._is_weird():
            self.thread_plan.SetPlanComplete(True)
            return True
        else:
            #           print('continued at 0x{:08x}'.format(this_frame.GetPC()))
            return False

    def should_step(self):
        return True


class StepInt(object):
    def __init__(self, thread_plan, dict_):
        self.thread_plan = thread_plan
        self.start_address = thread_plan.GetThread().GetFrameAtIndex(0).GetPC()

    def explains_stop(self, event):
        # We are stepping, so if we stop for any other reason, it isn't
        # because of us.
        if (
            self.thread_plan.GetThread().GetStopReason()
            == lldb.eStopReasonTrace
        ):
            return True
        else:
            return False

    def should_stop(self, event):
        this_frame = self.thread_plan.GetThread().GetFrameAtIndex(0)
        reg_groups = _get_reg_groups(this_frame)
        has_int_pending = _has_interrupt(reg_groups["System Registers"])

        if has_int_pending:
            self.thread_plan.SetPlanComplete(True)
            _print_ints(reg_groups)
            _print_vids(reg_groups)
            return True
        else:
            #           print('continued at 0x{:08x}'.format(this_frame.GetPC()))
            return False

    def should_step(self):
        return True


class StepIntRTE(object):
    def __init__(self, thread_plan, dict_):
        self.thread_plan = thread_plan
        self.start_address = thread_plan.GetThread().GetFrameAtIndex(0).GetPC()

    def explains_stop(self, event):
        # We are stepping, so if we stop for any other reason, it isn't
        # because of us.
        if (
            self.thread_plan.GetThread().GetStopReason()
            == lldb.eStopReasonTrace
        ):
            return True
        else:
            return False

    def should_stop(self, event):
        this_frame = self.thread_plan.GetThread().GetFrameAtIndex(0)
        reg_groups = _get_reg_groups(this_frame)
        in_handler = _is_in_handler(reg_groups["System Registers"])

        if not in_handler:
            self.thread_plan.SetPlanComplete(True)
            return True
        else:
            #           print('continued at 0x{:08x}'.format(this_frame.GetPC()))
            return False

    def should_step(self):
        return True


class StepModeChCommand(object):
    program = "stepmode"

    @classmethod
    def register_lldb_command(cls, debugger, module_name):
        parser = cls.create_options()
        cls.__doc__ = parser.format_help()
        # Add any commands contained in this module to LLDB
        cmd = "command script add -c %s.%s %s" % (
            module_name,
            cls.__name__,
            cls.program,
        )
        debugger.HandleCommand(cmd)
        print(
            'The "{0}" command has been installed, type "help {0}" or "{0} '
            '--help" for detailed help.'.format(cls.program)
        )

    @classmethod
    def create_options(cls):
        usage = "usage: %prog [options]"
        description = "TODO accept args int vs exception"

        # Pass add_help_option = False, since this keeps the command in line
        #  with lldb commands, and we wire up "help command" to work by
        # providing the long & short help methods below.
        parser = optparse.OptionParser(
            description=description,
            prog=cls.program,
            usage=usage,
            add_help_option=False,
        )

        parser.add_option(
            "-t",
            "--tbd",
            action="store_true",
            dest="tbd",
            help="tbd = True",
            default=True,
        )

        return parser

    def get_short_help(self):
        return "Example command for use in debugging"

    def get_long_help(self):
        return self.help_string

    def __init__(self, debugger, unused):
        self.parser = self.create_options()
        self.help_string = self.parser.format_help()

    def __call__(self, debugger, command, exe_ctx, result):
        # Use the Shell Lexer to properly parse up command options just like a
        # shell would
        command_args = shlex.split(command)

        try:
            (options, args) = self.parser.parse_args(command_args)
        except:
            # if you don't handle exceptions, passing an incorrect argument to
            # the OptionParser will cause LLDB to exit (courtesy of OptParse
            # dealing with argument errors by throwing SystemExit)
            result.SetError("option parsing failed")
            return

        # Always get program state from the lldb.SBExecutionContext passed
        # in as exe_ctx
        frame = exe_ctx.GetFrame()
        if not frame.IsValid():
            result.SetError("invalid frame")
            return

        cmd = "thread step-scripted -C HexQEMU.StepModeChange"
        debugger.HandleCommand(cmd)


class StepToIntRTECommand(object):
    program = "stepintrte"

    @classmethod
    def register_lldb_command(cls, debugger, module_name):
        parser = cls.create_options()
        cls.__doc__ = parser.format_help()
        # Add any commands contained in this module to LLDB
        cmd = "command script add -c %s.%s %s" % (
            module_name,
            cls.__name__,
            cls.program,
        )
        debugger.HandleCommand(cmd)
        print(
            'The "{0}" command has been installed, type "help {0}" or "{0} '
            '--help" for detailed help.'.format(cls.program)
        )

    @classmethod
    def create_options(cls):
        usage = "usage: %prog [options]"
        description = "TODO accept args int vs exception"

        # Pass add_help_option = False, since this keeps the command in line
        #  with lldb commands, and we wire up "help command" to work by
        # providing the long & short help methods below.
        parser = optparse.OptionParser(
            description=description,
            prog=cls.program,
            usage=usage,
            add_help_option=False,
        )

        return parser

    def get_short_help(self):
        return "Step to interrupt RTE"

    def get_long_help(self):
        return self.help_string

    def __init__(self, debugger, unused):
        self.parser = self.create_options()
        self.help_string = self.parser.format_help()

    def __call__(self, debugger, command, exe_ctx, result):
        # Use the Shell Lexer to properly parse up command options just like a
        # shell would
        command_args = shlex.split(command)

        try:
            (options, args) = self.parser.parse_args(command_args)
        except:
            # if you don't handle exceptions, passing an incorrect argument to
            # the OptionParser will cause LLDB to exit (courtesy of OptParse
            # dealing with argument errors by throwing SystemExit)
            result.SetError("option parsing failed")
            return

        # Always get program state from the lldb.SBExecutionContext passed
        # in as exe_ctx
        frame = exe_ctx.GetFrame()
        if not frame.IsValid():
            result.SetError("invalid frame")
            return

        cmd = "thread step-scripted -C HexQEMU.StepIntRTE"
        debugger.HandleCommand(cmd)


class StepToIntCommand(object):
    program = "stepint"

    @classmethod
    def register_lldb_command(cls, debugger, module_name):
        parser = cls.create_options()
        cls.__doc__ = parser.format_help()
        # Add any commands contained in this module to LLDB
        cmd = "command script add -c %s.%s %s" % (
            module_name,
            cls.__name__,
            cls.program,
        )
        debugger.HandleCommand(cmd)
        print(
            'The "{0}" command has been installed, type "help {0}" or "{0} '
            '--help" for detailed help.'.format(cls.program)
        )

    @classmethod
    def create_options(cls):
        usage = "usage: %prog [options]"
        description = "TODO accept args int vs exception"

        # Pass add_help_option = False, since this keeps the command in line
        #  with lldb commands, and we wire up "help command" to work by
        # providing the long & short help methods below.
        parser = optparse.OptionParser(
            description=description,
            prog=cls.program,
            usage=usage,
            add_help_option=False,
        )

        parser.add_option(
            "-t",
            "--tbd",
            action="store_true",
            dest="tbd",
            help="tbd = True",
            default=True,
        )

        return parser

    def get_short_help(self):
        return "Example command for use in debugging"

    def get_long_help(self):
        return self.help_string

    def __init__(self, debugger, unused):
        self.parser = self.create_options()
        self.help_string = self.parser.format_help()

    def __call__(self, debugger, command, exe_ctx, result):
        # Use the Shell Lexer to properly parse up command options just like a
        # shell would
        command_args = shlex.split(command)

        try:
            (options, args) = self.parser.parse_args(command_args)
        except:
            # if you don't handle exceptions, passing an incorrect argument to
            # the OptionParser will cause LLDB to exit (courtesy of OptParse
            # dealing with argument errors by throwing SystemExit)
            result.SetError("option parsing failed")
            return

        # Always get program state from the lldb.SBExecutionContext passed
        # in as exe_ctx
        frame = exe_ctx.GetFrame()
        if not frame.IsValid():
            result.SetError("invalid frame")
            return

        cmd = "thread step-scripted -C HexQEMU.StepInt"
        debugger.HandleCommand(cmd)


def __lldb_init_module(debugger, dict):
    # Register all classes that have a register_lldb_command method
    for _name, cls in inspect.getmembers(sys.modules[__name__]):
        if inspect.isclass(cls) and callable(
            getattr(cls, "register_lldb_command", None)
        ):
            cls.register_lldb_command(debugger, __name__)
