#![no_std]

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait ExperienceContract {
    #[init]
    fn init(&self) {
        let owner = self.blockchain().get_caller();
        self.managers().insert(owner);
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

        for (address, points) in self.members().iter() {
            members_multi.push((address, points).into());
        }

        members_multi.into()
    }

    #[endpoint(addPoints)]
    fn add_points_endpoint(&self, address: ManagedAddress, amount: BigUint) {
        self.require_caller_is_manager();

        let current_points = self.members().get(&address).unwrap_or_default();
        let new_points = current_points + amount;

        self.members().insert(address, new_points);
    }

    #[endpoint(removePoints)]
    fn remove_points_endpoint(&self, address: ManagedAddress, amount: BigUint) {
        self.require_caller_is_manager();

        let current_points = self.members().get(&address).unwrap_or_default();

        if amount > current_points {
            self.members().remove(&address);
            return;
        }

        self.members().insert(address, current_points - amount);
    }

    fn require_caller_is_manager(&self) {
        let caller = self.blockchain().get_caller();

        require!(self.managers().contains(&caller), "caller must be manager");
    }

    #[view(getManagers)]
    #[storage_mapper("managers")]
    fn managers(&self) -> UnorderedSetMapper<ManagedAddress>;

    #[storage_mapper("members")]
    fn members(&self) -> MapMapper<ManagedAddress, BigUint>;
}
