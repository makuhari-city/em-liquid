mod liquid_democracy;

use liquid_democracy::{LDResult, LiquidDemocracy};
use vote::VoteData;

pub async fn calculate(info: VoteData) -> LDResult {
    let liq = LiquidDemocracy::new(info);
    liq.calculate().await
}
