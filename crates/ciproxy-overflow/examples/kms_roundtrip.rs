//! Live Cloud KMS round trip. Arguments carry credential paths only.

mod live_support;

use std::{path::PathBuf, sync::Arc};

use async_trait::async_trait;
use ciproxy_overflow::crypto::{KeyCipher, KeyContext, SecretPurpose};
use ciproxy_overflow::error::Result;
use ciproxy_overflow::kms::cipher::CloudKmsCipher;
use ciproxy_overflow::kms::transport::{AccessTokens, CloudKmsRest};
use ciproxy_overflow::secrets::SecretBytes;
use ciproxy_protocol::secret::Secret;
use clap::Parser;
use uuid::Uuid;

use live_support::SafeResult;

#[derive(Parser)]
struct Arguments {
    #[arg(long)]
    boat_key_file: PathBuf,
    #[arg(long)]
    access_token_file: PathBuf,
    #[arg(long)]
    kms_key_resource: String,
}

struct FileToken(Secret);

#[async_trait]
impl AccessTokens for FileToken {
    async fn access_token(&self) -> Result<Secret> {
        Ok(self.0.clone())
    }
}

#[tokio::main]
async fn main() -> live_support::Result<()> {
    let args = Arguments::parse();
    let token = live_support::secret(&args.access_token_file)?;
    let key = live_support::secret(&args.boat_key_file)?;
    let transport = CloudKmsRest::new(args.kms_key_resource, Arc::new(FileToken(token)))
        .safe("configure KMS")?;
    let cipher: &dyn KeyCipher = &CloudKmsCipher::new(Arc::new(transport));
    let context = KeyContext {
        workspace_id: Uuid::new_v4(),
        id: Uuid::new_v4(),
        purpose: SecretPurpose::BoatKey,
    };
    let ciphertext = cipher
        .seal(
            context,
            SecretBytes::new(key.expose_secret().as_bytes().to_vec()),
        )
        .await
        .safe("KMS encrypt")?;
    let opened = cipher.open(context, ciphertext).await.safe("KMS decrypt")?;
    if opened.expose() != key.expose_secret().as_bytes() {
        return Err(std::io::Error::other("round trip mismatch")).safe("verify KMS round trip");
    }
    println!("Cloud KMS round trip passed.");
    Ok(())
}
