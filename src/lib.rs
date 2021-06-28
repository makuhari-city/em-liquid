mod liquid_democracy;

use liquid_democracy::{LDResult, LiquidDemocracy};
use vote::TopicInfo;

pub fn calculate(info: TopicInfo) -> LDResult {
    let liq = LiquidDemocracy::new(info);
    liq.calculate()
}
