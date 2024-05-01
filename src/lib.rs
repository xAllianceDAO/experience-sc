#![no_std]

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait Plug {
    #[init]
    fn init(&self, manager: ManagedAddress) {
        self.manager().set(manager);
    }

    #[upgrade]
    fn upgrade(&self) {}

    #[view(getDaoVoteWeight)]
    fn get_dao_vote_weight_view(
        &self,
        address: ManagedAddress,
        token: OptionalValue<TokenIdentifier>,
    ) -> BigUint {
        self.members().get(&address).unwrap_or_default()
    }

    #[view(getDaoMembers)]
    fn get_dao_members_view(
        &self,
        token: OptionalValue<TokenIdentifier>,
    ) -> MultiValueEncoded<MultiValue2<ManagedAddress, BigUint>> {
        let mut members_multi = MultiValueEncoded::new();

        for (address, weight) in self.members().iter() {
            members_multi.push((address, weight).into());
        }

        members_multi.into()
    }

    #[endpoint(increaseExperience)]
    fn add_member_endpoint(&self, address: ManagedAddress, weight: BigUint) {
        self.require_caller_is_manager();

        let current_weight = self.members().get(&address).unwrap_or_default();
        let new_weight = current_weight + weight;

        self.members().insert(address, new_weight);
    }

    #[endpoint(decreaseExperience)]
    fn remove_member_endpoint(&self, address: ManagedAddress, weight: BigUint) {
        self.require_caller_is_manager();

        let current_weight = self.members().get(&address).unwrap_or_default();
        let new_weight = if current_weight > weight {
            current_weight - weight
        } else {
            BigUint::zero()
        };

        self.members().insert(address, new_weight);
    }

    fn require_caller_is_manager(&self) {
        let caller = self.blockchain().get_caller();
        let manager = self.manager().get();

        require!(caller == manager, "caller must be manager");
    }

    #[storage_mapper("manager")]
    fn manager(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("members")]
    fn members(&self) -> MapMapper<ManagedAddress, BigUint>;
}
