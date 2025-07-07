#![no_std]

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[type_abi]
#[derive(TopEncode, TopDecode, Clone)]
pub struct Tier<M: ManagedTypeApi> {
    pub name: ManagedBuffer<M>,
    pub min_threshold: BigUint<M>,
    pub voting_power: BigUint<M>,
}

#[multiversx_sc::contract]
pub trait ExperienceContract {
    #[init]
    fn init(&self) {
        let owner = self.blockchain().get_caller();
        self.managers().insert(owner);
    }

    #[upgrade]
    fn upgrade(&self) {}

    #[only_owner]
    #[endpoint(addManager)]
    fn add_manager_endpoint(&self, address: ManagedAddress) {
        self.managers().insert(address);
    }

    #[only_owner]
    #[endpoint(removeManager)]
    fn remove_manager_endpoint(&self, address: ManagedAddress) {
        self.managers().swap_remove(&address);
    }

    #[view(getDaoVoteWeight)]
    fn get_dao_vote_weight_view(
        &self,
        address: ManagedAddress,
        _token: OptionalValue<TokenIdentifier>,
    ) -> BigUint {
        self.get_member_tier_internal(&address)
            .map_or_else(BigUint::zero, |tier| tier.voting_power)
    }

    #[view(getDaoMembers)]
    fn get_dao_members_view(
        &self,
        _token: OptionalValue<TokenIdentifier>,
    ) -> MultiValueEncoded<MultiValue2<ManagedAddress, BigUint>> {
        self.members()
            .iter()
            .map(|(address, points)| (address, points).into())
            .collect()
    }

    #[endpoint(addPoints)]
    fn add_points_endpoint(&self, address: ManagedAddress, amount: BigUint) {
        self.require_caller_is_manager();
        let new_points = self.members().get(&address).unwrap_or_default() + amount;
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

    #[endpoint(addTier)]
    fn add_tier_endpoint(
        &self,
        name: ManagedBuffer,
        min_threshold: BigUint,
        voting_power: BigUint,
    ) {
        self.require_caller_is_manager();
        self.tiers().push(&Tier {
            name,
            min_threshold,
            voting_power,
        });
    }

    #[endpoint(clearAllTiers)]
    fn clear_all_tiers_endpoint(&self) {
        self.require_caller_is_manager();
        self.tiers().clear();
    }

    #[view(getMemberPoints)]
    fn get_member_points_view(&self, address: ManagedAddress) -> BigUint {
        self.members().get(&address).unwrap_or_default()
    }

    #[view(getMemberTier)]
    fn get_member_tier_view(
        &self,
        address: ManagedAddress,
    ) -> OptionalValue<MultiValue3<ManagedBuffer, BigUint, BigUint>> {
        self.get_member_tier_internal(&address)
            .map(|t| (t.name, t.min_threshold, t.voting_power).into())
            .into()
    }

    #[view(getAllTiers)]
    fn get_all_tiers_view(
        &self,
    ) -> MultiValueEncoded<MultiValue3<ManagedBuffer, BigUint, BigUint>> {
        self.tiers()
            .iter()
            .map(|tier| (tier.name, tier.min_threshold, tier.voting_power).into())
            .collect()
    }

    fn get_member_tier_internal(&self, address: &ManagedAddress) -> Option<Tier<Self::Api>> {
        let member_points = self.members().get(address).unwrap_or_default();
        let mut best_tier = None;
        let mut best_threshold = BigUint::zero();

        for tier in self.tiers().iter() {
            if member_points < tier.min_threshold {
                continue;
            }

            if tier.min_threshold > best_threshold {
                best_threshold = tier.min_threshold.clone();
                best_tier = Some(tier.clone());
            }
        }

        best_tier
    }

    fn require_caller_is_manager(&self) {
        require!(
            self.managers().contains(&self.blockchain().get_caller()),
            "caller must be manager"
        );
    }

    #[view(getManagers)]
    #[storage_mapper("managers")]
    fn managers(&self) -> UnorderedSetMapper<ManagedAddress>;

    #[storage_mapper("members")]
    fn members(&self) -> MapMapper<ManagedAddress, BigUint>;

    #[storage_mapper("tiers")]
    fn tiers(&self) -> VecMapper<Tier<Self::Api>>;
}
