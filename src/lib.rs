use soroban_sdk::{contract, contractimpl, symbol, vec, Address, Env, Map, Symbol, Vec};

#[contract]
pub struct SecurityRegistry;

#[contractimpl]
impl SecurityRegistry {
    pub fn register_address(env: Env, address: Address, is_suspicious: bool) {
        let key = symbol!("suspicious");
        let mut map: Map<Address, bool> = env.storage().instance().get(&key).unwrap_or(Map::new(&env));
        map.set(address, is_suspicious);
        env.storage().instance().set(&key, &map);
    }

    pub fn is_suspicious(env: Env, address: Address) -> bool {
        let key = symbol!("suspicious");
        let map: Map<Address, bool> = env.storage().instance().get(&key).unwrap_or(Map::new(&env));
        map.get(address).unwrap_or(false)
    }

    pub fn register_trusted(env: Env, address: Address) {
        let key = symbol!("trusted");
        let mut map: Map<Address, bool> = env.storage().instance().get(&key).unwrap_or(Map::new(&env));
        map.set(address, true);
        env.storage().instance().set(&key, &map);
    }

    pub fn is_trusted(env: Env, address: Address) -> bool {
        let key = symbol!("trusted");
        let map: Map<Address, bool> = env.storage().instance().get(&key).unwrap_or(Map::new(&env));
        map.get(address).unwrap_or(false)
    }

    pub fn get_flags(env: Env, address: Address) -> (bool, bool) {
        let suspicious_key = symbol!("suspicious");
        let trusted_key = symbol!("trusted");

        let suspicious_map: Map<Address, bool> = env.storage().instance().get(&suspicious_key).unwrap_or(Map::new(&env));
        let trusted_map: Map<Address, bool> = env.storage().instance().get(&trusted_key).unwrap_or(Map::new(&env));

        let is_suspicious = suspicious_map.get(address).unwrap_or(false);
        let is_trusted = trusted_map.get(address).unwrap_or(false);

        (is_suspicious, is_trusted)
    }
}

mod test;
