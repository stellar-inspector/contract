use soroban_sdk::{contract, contractimpl, Address, Env, Map, Symbol};

#[contract]
pub struct TestSecurityRegistry;

#[contractimpl]
impl TestSecurityRegistry {
    pub fn test_register_and_check(env: Env, address: Address) {
        let key = symbol!("suspicious");
        let mut map: Map<Address, bool> = env.storage().instance().get(&key).unwrap_or(Map::new(&env));
        map.set(address.clone(), true);
        env.storage().instance().set(&key, &map);

        let result = SecurityRegistry::is_suspicious(env.clone(), address);
        assert!(result);
    }
}
