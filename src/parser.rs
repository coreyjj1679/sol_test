use regex::Regex;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug)]
pub struct TokenTransfer {
    #[serde(rename = "fromTokenAccount")]
    pub from_token_account: String,
    #[serde(rename = "fromUserAccount")]
    pub from_user_account: String,
    pub mint: String,
    #[serde(rename = "toTokenAccount")]
    pub to_token_account: String,
    #[serde(rename = "toUserAccount")]
    pub to_user_account: String,
    #[serde(rename = "tokenAmount")]
    pub token_amount: f64,
    #[serde(rename = "tokenStandard")]
    pub token_standard: String,
}

#[derive(Debug)]
pub struct TransactionMeta {
    pub sender: String,
    pub from_amount: f64,
    pub to_amount: f64,
    pub from_token: String,
    pub to_token: String,
}

#[derive(Debug)]
pub struct Transaction {
    pub signature: String,
    pub block_slot: u64,
    pub timestamp: u64,
    pub amm: String,
    pub sender: String,
    pub from_amount: f64,
    pub to_amount: f64,
    pub from_token: String,
    pub to_token: String,
}

pub fn find_mint_by_token_amount(transfers: &[TokenTransfer], amount: f64) -> Option<String> {
    // Iterate through the transfers to find the mint for the specified token amount
    for transfer in transfers {
        if (transfer.token_amount - amount).abs() < f64::EPSILON {
            return Some(transfer.mint.clone());
        }
    }
    None
}

pub fn parse_description(input: &str) -> Option<TransactionMeta> {
    // Define a regex pattern to match the desired components
    let pattern = r"(?P<sender>[A-Za-z0-9]+) swapped (?P<from_amount>[\d\.]+) (?P<from_token>[A-Z]+) for (?P<to_amount>[\d\.]+) (?P<to_token>[A-Z]+)";
    let re = Regex::new(pattern).unwrap();

    // Use the regex to capture groups from the input string
    if let Some(captures) = re.captures(input) {
        let sender = captures.name("sender").unwrap().as_str().to_string();
        let from_amount: f64 = captures
            .name("from_amount")
            .unwrap()
            .as_str()
            .parse()
            .ok()?;
        let to_amount: f64 = captures.name("to_amount").unwrap().as_str().parse().ok()?;
        let from_token = captures.name("from_token").unwrap().as_str().to_string();
        let to_token = captures.name("to_token").unwrap().as_str().to_string();

        Some(TransactionMeta {
            sender,
            from_amount,
            to_amount,
            from_token,
            to_token,
        })
    } else {
        None
    }
}

pub fn parse_transaction(intput: &str) -> Option<Transaction> {
    let parsed: Value = serde_json::from_str(intput).expect("Failed to parse JSON");
    let parsed_transfer = parsed.get(0).expect("Failed to parse transfer");

    let description = parsed_transfer
        .get("description")
        .unwrap()
        .as_str()
        .unwrap();
    let transaction = parse_description(description).unwrap();

    let transfers: Vec<TokenTransfer> =
        serde_json::from_value(parsed_transfer["tokenTransfers"].clone())
            .expect("Failed to parse tokenTransfers");

    let from_token = find_mint_by_token_amount(&transfers, transaction.from_amount)
        .unwrap()
        .to_string();
    let to_token = find_mint_by_token_amount(&transfers, transaction.to_amount)
        .unwrap()
        .to_string();

    let timestamp: u64 = parsed_transfer
        .get("timestamp")
        .unwrap()
        .to_string()
        .parse()
        .ok()?;
    let block_slot: u64 = parsed_transfer
        .get("slot")
        .unwrap()
        .to_string()
        .parse()
        .ok()?;
    let amm = parsed_transfer.get("source").unwrap().as_str().unwrap();
    let signature = parsed_transfer.get("signature").unwrap().as_str().unwrap();

    Some(Transaction {
        signature: signature.to_string(),
        block_slot,
        timestamp,
        amm: amm.to_string(),
        sender: transaction.sender,
        from_amount: transaction.from_amount,
        to_amount: transaction.to_amount,
        from_token,
        to_token,
    })
}
