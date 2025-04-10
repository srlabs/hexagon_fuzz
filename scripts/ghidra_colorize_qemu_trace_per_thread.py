# Import necessary Ghidra modules
from ghidra.app.plugin.core.colorizer import ColorizingService
from ghidra.program.model.address import Address
from ghidra.program.model.address import AddressSet
from ghidra.program.model.block import BasicBlockModel
from ghidra.app.script import GhidraScript
from java.awt import Color
import os

# Function to parse the QEMU trace file and return a dictionary of trace numbers and addresses
def parse_qemu_trace(trace_file_path):
    trace_dict = {}
    try:
        with open(trace_file_path, 'r') as file:
            for line in file:
                line = line.strip()
                if "Trace" in line:
                    parts = line.split(':')
                    trace_num = int(parts[0].split()[1])
                    start_idx = line.index('[') + 1
                    end_idx = line.index(']')
                    full_address = line[start_idx:end_idx].split('/')[1]
                    address_32bit = full_address[-8:]

                    if trace_num not in trace_dict:
                        trace_dict[trace_num] = []
                    trace_dict[trace_num].append(address_32bit)
    except IOError as e:
        print("Failed to read trace file: {}".format(e))
    return trace_dict

# Function to get the instruction package for a given address
# Import necessary Ghidra modules
from ghidra.app.plugin.core.colorizer import ColorizingService
from ghidra.program.model.address import Address
from ghidra.program.model.address import AddressSet
from ghidra.program.model.block import BasicBlockModel
from ghidra.app.script import GhidraScript
from java.awt import Color
import os

# Function to parse the QEMU trace file and return a dictionary of trace numbers and addresses
def parse_qemu_trace(trace_file_path):
    trace_dict = {}
    try:
        with open(trace_file_path, 'r') as file:
            for line in file:
                line = line.strip()
                if "Trace" in line:
                    parts = line.split(':')
                    trace_num = int(parts[0].split()[1])
                    start_idx = line.index('[') + 1
                    end_idx = line.index(']')
                    full_address = line[start_idx:end_idx].split('/')[1]
                    address_32bit = full_address[-8:]

                    if trace_num not in trace_dict:
                        trace_dict[trace_num] = []
                    trace_dict[trace_num].append(address_32bit)
    except IOError as e:
        print("Failed to read trace file: {}".format(e))
    return trace_dict

# Function to get the instruction package for a given address
def get_instruction_package(address):
    block_model = BasicBlockModel(currentProgram)
    package_addresses = []

    try:
        block = block_model.getFirstCodeBlockContaining(address, monitor)
        if block:
            package_addresses = list(block.getAddresses(True))  # True for 'forward'
    except Exception as e:
        print("Failed to get instruction package for address {}: {}".format(address, e))

    return package_addresses

# Function to highlight instruction packages
def highlight_instruction_packages(trace_dict, color_map):
    service = state.getTool().getService(ColorizingService)
    if service is None:
        print("Can't find ColorizingService service")
        return

    for trace_num, addresses in trace_dict.items():
        color = color_map.get(trace_num, Color.BLACK)  # Default to black if no color specified
        address_set = AddressSet()
        for address_str in addresses:
            try:
                address = currentProgram.getAddressFactory().getAddress(address_str)
                if address:
                    package_addresses = get_instruction_package(address)
                    for pkg_address in package_addresses:
                        address_set.add(pkg_address)
            except Exception as e:
                print("Failed to highlight address {}: {}".format(address_str, e))

        service.setBackgroundColor(address_set, color)

# Main function
def main():
    script_dir = os.path.dirname(os.path.realpath(__file__))
    trace_file_path = os.path.join(script_dir, "trace.txt")
    trace_dict = parse_qemu_trace(trace_file_path)

    # Define colors for each trace number
    color_map = {
        0: Color.GREEN,
        1: Color.BLUE,
        2: Color.RED,
        3: Color.YELLOW,
        4: Color.ORANGE,
        5: Color.CYAN
    }

    highlight_instruction_packages(trace_dict, color_map)
    print("Highlighted instruction packages for {} traces from QEMU trace.".format(len(trace_dict)))

if __name__ == "__main__":
    main()