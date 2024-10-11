from communex._common import get_node_url
from communex.client import CommuneClient
from communex.compat.key import Ss58Address, Keypair
from typing import Dict, Tuple
import subprocess
import json
import time
comx = CommuneClient(get_node_url())

def get_keypair(key_name: str="vali::eden") -> Keypair:
    
    result = subprocess.run(["/home/administrator/repos/module_validator_rust/modules/cli-wrapper/target/debug/cli_wrapper", key_name, "decrypt"], check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    with open(f"/home/administrator/.commune/key/{key_name}.json", "r", encoding="utf-8") as f:
        json_data = json.loads(f.read())
        data = json.loads(json_data["data"])
        private_key = data["private_key"]
        public_key = data["public_key"]
        ss58_address = data["ss58_address"]
    subprocess.run(["/home/administrator/repos/module_validator_rust/modules/cli-wrapper/target/debug/cli_wrapper", key_name, "encrypt"], check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    return Keypair(ss58_address=ss58_address, public_key=public_key, private_key=private_key)


def get_query_maps(subnet=10) -> Tuple[Dict[str, str], Dict[str, str], Dict[str, str]]:
    address_map = comx.query_map_address(subnet)
    weights_map = comx.query_map_weights(subnet)
    ss58key_map = comx.query_map_key(subnet)
    return address_map, weights_map, ss58key_map

SYNTHIA_VALIDATOR = "5FjUVoQCdAc9sGei7dVxtR8jnbf656CrWn4dnH8yoTWxXERs"

VALI_KEY = get_keypair(key_name="vali::eden")

def get_all_query_maps() -> Tuple[Dict[str, dict], Dict[str, dict], Dict[str, dict]]:
    subnets = [3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20]
    address_maps = {}
    weights_maps = {}
    ss58key_maps = {}
    for subnet in subnets:
        address_maps[f"{subnet}"] = {}
        weights_maps[f"{subnet}"] = {}
        ss58key_maps[f"{subnet}"] = {}
        address_map, weights_map, ss58key_map = get_query_maps(subnet)
        address_maps[f"{subnet}"] = address_map
        weights_maps[f"{subnet}"] = weights_map
        ss58key_maps[f"{subnet}"] = ss58key_map
    return address_maps, weights_maps, ss58key_maps

ADDRESS_MAPS, WEIGHTS_MAPS, SS58KEY_MAPS = get_all_query_maps()

def get_ss58key_info(subnet=10, ss58key=SYNTHIA_VALIDATOR) -> Dict[str, int]:
    uid = SS58KEY_MAPS[f"{subnet}"][ss58key]
    weights = WEIGHTS_MAPS[f"{subnet}"][uid]
    return uid, weights


def main():
    for subnet in ADDRESS_MAPS.keys():
        self_uid, _ = get_ss58key_info()
        uids = []
        weights = []
        _, synthia_weights = get_ss58key_info(subnet=subnet)
        
        for uid, weight in synthia_weights.items():
            if uid != self_uid:
                continue
            uids.append(uid)
            weights.append(weight)
        print(f"Subnet {subnet}: {uids} {weights}")
        
        reciept = comx.vote(VALI_KEY, uids, weights, subnet)
        if reciept.is_success:
            print(f"Vote success on subnet {subnet}: {reciept.extrinsic}")
        else:
            print(f"Vote failed on subnet {subnet}: {reciept.error_message}")

if __name__ == "__main__":
    while True:
        main()
        time.sleep(60)
        ADDRESS_MAPS, WEIGHTS_MAPS, SS58KEY_MAPS = get_all_query_maps()
    


    