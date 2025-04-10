# Import necessary Ghidra modules
from ghidra.app.plugin.core.colorizer import ColorizingService
from ghidra.program.model.address import Address
from ghidra.program.model.address import AddressSet
from ghidra.program.model.block import BasicBlockModel
from ghidra.app.script import GhidraScript
from java.awt import Color
import os

# Function to parse the QEMU trace file and return a list of addresses
def parse_qemu_trace(trace_file_path):
    addresses = []
    try:
        with open(trace_file_path, 'r') as file:
            for line in file:
                line = line.strip()
                if "Trace" in line:
                    # Extract the address within the brackets
                    start_idx = line.index('[') + 1
                    end_idx = line.index(']')
                    full_address = line[start_idx:end_idx].split('/')[1]
                    # Extract the lower 32 bits
                    address_32bit = full_address[-8:]  # Get last 8 characters (32 bits in hex)
                    addresses.append(address_32bit)
    except IOError as e:
        print("Failed to read trace file: {}".format(e))
    return addresses

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
def highlight_instruction_packages(addresses, color):
    service = state.getTool().getService(ColorizingService)
    if service is None:
        print("Can't find ColorizingService service")
        return

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
    addresses = parse_qemu_trace(trace_file_path)
    highlight_color = Color.GREEN  # You can choose any color you like
    highlight_instruction_packages(addresses, highlight_color)
    print("Highlighted instruction packages for {} addresses from QEMU trace.".format(len(addresses)))

if __name__ == "__main__":
    main()
