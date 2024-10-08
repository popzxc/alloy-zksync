use alloy::providers::fillers::{
    ChainIdFiller, GasFiller, JoinFill, NonceFiller, RecommendedFillers,
};
use alloy::providers::Provider;
use alloy::transports::{BoxTransport, Transport};

use crate::network::Zksync;

#[async_trait::async_trait]
pub trait ZksyncProvider<T = BoxTransport>: Provider<T, Zksync>
where
    T: Transport + Clone,
{
}

impl RecommendedFillers for Zksync {
    type RecomendedFillers = JoinFill<GasFiller, JoinFill<NonceFiller, ChainIdFiller>>;

    fn recommended_fillers() -> Self::RecomendedFillers {
        JoinFill::new(
            GasFiller,
            JoinFill::new(NonceFiller::default(), ChainIdFiller::default()),
        )
    }
}
