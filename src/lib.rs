mod liquid_democracy;

use liquid_democracy::{LDResult, LiquidDemocracy};
use vote::VoteInfo;

pub async fn calculate(info: VoteInfo) -> LDResult {
    let liq = LiquidDemocracy::new(info);
    liq.calculate().await
}
