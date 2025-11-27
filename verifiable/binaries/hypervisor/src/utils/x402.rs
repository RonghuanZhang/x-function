use x402_axum::{facilitator_client::FacilitatorClient, PriceTag, X402Middleware};
use x402_rs::{
    network::{Network, USDCDeployment},
    types::{EvmAddress, MoneyAmount, TokenAmount},
};

pub const X402_FACILITATOR_URL: &str = "https://x402.org/facilitator/";
pub const X402_FIXED_PRICE_USDC: &str = "0.01";
pub const X402_PAY_RECEIPIENT: &str = "0xfa4c85133b817e0cefce87b6393841ef45d25ac4";

pub fn create_x402_middleware(price: &str) -> X402Middleware<FacilitatorClient> {
    let usdc = USDCDeployment::by_network(Network::BaseSepolia);
    let price_amount: TokenAmount = price
        .parse::<MoneyAmount>()
        .and_then(|a| a.as_token_amount(usdc.decimals.into()))
        .expect("valid x402 price amount");
    let recipient: EvmAddress = X402_PAY_RECEIPIENT
        .parse()
        .expect("x402 recipient eth address");
    let price_tag = PriceTag::new(recipient, price_amount, usdc);

    X402Middleware::try_from(X402_FACILITATOR_URL)
        .expect("valid x402 facilitator url")
        .with_price_tag(vec![price_tag])
}
