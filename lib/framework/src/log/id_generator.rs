use uuid::Uuid;

pub fn random_id() -> String {
    let bytes = Uuid::now_v7();
    let bytes = bytes.as_bytes();
    bs58::encode(bytes).into_string()
}
