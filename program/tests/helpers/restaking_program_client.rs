#[allow(dead_code)]
use jito_bytemuck::AccountDeserialize;
use jito_restaking_core::{
    config::Config, ncn::Ncn, ncn_operator_state::NcnOperatorState,
    ncn_vault_slasher_ticket::NcnVaultSlasherTicket, ncn_vault_ticket::NcnVaultTicket,
    operator::Operator, operator_vault_ticket::OperatorVaultTicket,
};
use jito_restaking_sdk::{
    instruction::OperatorAdminRole,
    sdk::{
        cooldown_ncn_vault_ticket, initialize_config, initialize_ncn,
        initialize_ncn_operator_state, initialize_ncn_vault_slasher_ticket,
        initialize_ncn_vault_ticket, initialize_operator, initialize_operator_vault_ticket,
        ncn_cooldown_operator, ncn_set_admin, ncn_warmup_operator, operator_cooldown_ncn,
        operator_set_admin, operator_set_fee, operator_set_secondary_admin, operator_warmup_ncn,
        set_config_admin, warmup_ncn_vault_slasher_ticket, warmup_ncn_vault_ticket,
        warmup_operator_vault_ticket,
    },
};
use solana_commitment_config::CommitmentLevel;
use solana_keypair::Keypair;
use solana_native_token::sol_str_to_lamports;
use solana_program_test::BanksClient;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_system_interface::instruction::transfer;
use solana_transaction::Transaction;

use super::TestError;

#[derive(Debug)]
#[allow(dead_code)]
pub struct NcnRoot {
    pub ncn_pubkey: Pubkey,
    pub ncn_admin: Keypair,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct OperatorRoot {
    pub operator_pubkey: Pubkey,
    pub operator_admin: Keypair,
}

pub struct RestakingProgramClient {
    banks_client: BanksClient,
    #[allow(dead_code)]
    payer: Keypair,
}

impl RestakingProgramClient {
    #[allow(dead_code)]
    pub const fn new(banks_client: BanksClient, payer: Keypair) -> Self {
        Self {
            banks_client,
            payer,
        }
    }

    #[allow(dead_code)]
    pub async fn get_ncn(&mut self, ncn: &Pubkey) -> Result<Ncn, TestError> {
        let account = self
            .banks_client
            .get_account_with_commitment(*ncn, CommitmentLevel::Processed)
            .await?
            .unwrap();

        Ok(*Ncn::try_from_slice_unchecked(account.data.as_slice())?)
    }

    #[allow(dead_code)]
    pub async fn get_config(&mut self, account: &Pubkey) -> Result<Config, TestError> {
        let account = self.banks_client.get_account(*account).await?.unwrap();
        Ok(*Config::try_from_slice_unchecked(account.data.as_slice())?)
    }

    #[allow(dead_code)]
    pub async fn get_ncn_vault_ticket(
        &mut self,
        ncn: &Pubkey,
        vault: &Pubkey,
    ) -> Result<NcnVaultTicket, TestError> {
        let account =
            NcnVaultTicket::find_program_address(&jito_restaking_program::id(), ncn, vault).0;
        let account = self.banks_client.get_account(account).await?.unwrap();
        Ok(*NcnVaultTicket::try_from_slice_unchecked(
            account.data.as_slice(),
        )?)
    }

    #[allow(dead_code)]
    pub async fn get_ncn_operator_state(
        &mut self,
        ncn: &Pubkey,
        operator: &Pubkey,
    ) -> Result<NcnOperatorState, TestError> {
        let account =
            NcnOperatorState::find_program_address(&jito_restaking_program::id(), ncn, operator).0;
        let account = self.banks_client.get_account(account).await?.unwrap();
        Ok(*NcnOperatorState::try_from_slice_unchecked(
            account.data.as_slice(),
        )?)
    }

    #[allow(dead_code)]
    pub async fn get_ncn_vault_slasher_ticket(
        &mut self,
        ncn: &Pubkey,
        vault: &Pubkey,
        slasher: &Pubkey,
    ) -> Result<NcnVaultSlasherTicket, TestError> {
        let account = NcnVaultSlasherTicket::find_program_address(
            &jito_restaking_program::id(),
            ncn,
            vault,
            slasher,
        )
        .0;
        let account = self.banks_client.get_account(account).await?.unwrap();
        Ok(*NcnVaultSlasherTicket::try_from_slice_unchecked(
            account.data.as_slice(),
        )?)
    }

    #[allow(dead_code)]
    pub async fn get_operator(&mut self, account: &Pubkey) -> Result<Operator, TestError> {
        let account = self.banks_client.get_account(*account).await?.unwrap();
        Ok(*Operator::try_from_slice_unchecked(
            account.data.as_slice(),
        )?)
    }

    #[allow(dead_code)]
    pub async fn get_operator_vault_ticket(
        &mut self,
        operator: &Pubkey,
        vault: &Pubkey,
    ) -> Result<OperatorVaultTicket, TestError> {
        let account = OperatorVaultTicket::find_program_address(
            &jito_restaking_program::id(),
            operator,
            vault,
        )
        .0;
        let account = self.banks_client.get_account(account).await?.unwrap();
        Ok(*OperatorVaultTicket::try_from_slice_unchecked(
            account.data.as_slice(),
        )?)
    }

    #[allow(dead_code)]
    pub async fn get_operator_ncn_ticket(
        &mut self,
        operator: &Pubkey,
        ncn: &Pubkey,
    ) -> Result<NcnOperatorState, TestError> {
        let account =
            NcnOperatorState::find_program_address(&jito_restaking_program::id(), operator, ncn).0;
        let account = self.banks_client.get_account(account).await?.unwrap();
        Ok(*NcnOperatorState::try_from_slice_unchecked(
            account.data.as_slice(),
        )?)
    }

    #[allow(dead_code)]
    pub async fn do_initialize_config(&mut self) -> Result<Keypair, TestError> {
        let restaking_config_pubkey = Config::find_program_address(&jito_restaking_program::id()).0;
        let restaking_config_admin = Keypair::new();

        self._airdrop(&restaking_config_admin.pubkey(), 1.0).await?;
        self.initialize_config(&restaking_config_pubkey, &restaking_config_admin)
            .await?;

        Ok(restaking_config_admin)
    }

    #[allow(dead_code)]
    pub async fn do_initialize_operator(&mut self) -> Result<OperatorRoot, TestError> {
        // create operator + add operator vault
        let operator_base = Keypair::new();
        let operator_pubkey =
            Operator::find_program_address(&jito_restaking_program::id(), &operator_base.pubkey())
                .0;
        let operator_admin = Keypair::new();
        self._airdrop(&operator_admin.pubkey(), 1.0).await?;
        self.initialize_operator(
            &Config::find_program_address(&jito_restaking_program::id()).0,
            &operator_pubkey,
            &operator_admin,
            &operator_base,
            0,
        )
        .await?;
        Ok(OperatorRoot {
            operator_pubkey,
            operator_admin,
        })
    }

    #[allow(dead_code)]
    pub async fn do_initialize_operator_vault_ticket(
        &mut self,
        operator_root: &OperatorRoot,
        vault_pubkey: &Pubkey,
    ) -> Result<(), TestError> {
        let operator_vault_ticket = OperatorVaultTicket::find_program_address(
            &jito_restaking_program::id(),
            &operator_root.operator_pubkey,
            vault_pubkey,
        )
        .0;
        self.initialize_operator_vault_ticket(
            &Config::find_program_address(&jito_restaking_program::id()).0,
            &operator_root.operator_pubkey,
            vault_pubkey,
            &operator_vault_ticket,
            &operator_root.operator_admin,
            &operator_root.operator_admin,
        )
        .await?;

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn do_warmup_operator_vault_ticket(
        &mut self,
        operator_root: &OperatorRoot,
        vault_pubkey: &Pubkey,
    ) -> Result<(), TestError> {
        let operator_vault_ticket = OperatorVaultTicket::find_program_address(
            &jito_restaking_program::id(),
            &operator_root.operator_pubkey,
            vault_pubkey,
        )
        .0;
        self.warmup_operator_vault_ticket(
            &Config::find_program_address(&jito_restaking_program::id()).0,
            &operator_root.operator_pubkey,
            vault_pubkey,
            &operator_vault_ticket,
            &operator_root.operator_admin,
        )
        .await
    }

    pub async fn warmup_operator_vault_ticket(
        &mut self,
        config: &Pubkey,
        operator: &Pubkey,
        vault: &Pubkey,
        operator_vault_ticket: &Pubkey,
        admin: &Keypair,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[warmup_operator_vault_ticket(
                &jito_restaking_program::id(),
                config,
                operator,
                vault,
                operator_vault_ticket,
                &admin.pubkey(),
            )],
            Some(&self.payer.pubkey()),
            &[admin, &self.payer],
            blockhash,
        ))
        .await
    }

    pub async fn initialize_config(
        &mut self,
        config: &Pubkey,
        config_admin: &Keypair,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self.process_transaction(&Transaction::new_signed_with_payer(
            &[initialize_config(
                &jito_restaking_program::id(),
                config,
                &config_admin.pubkey(),
                &jito_vault_program::id(),
            )],
            Some(&config_admin.pubkey()),
            &[config_admin],
            blockhash,
        ))
        .await
    }

    #[allow(dead_code)]
    pub async fn do_initialize_ncn(&mut self) -> Result<NcnRoot, TestError> {
        let ncn_admin = Keypair::new();
        let ncn_base = Keypair::new();

        self._airdrop(&ncn_admin.pubkey(), 1.0).await?;

        let ncn_pubkey =
            Ncn::find_program_address(&jito_restaking_program::id(), &ncn_base.pubkey()).0;
        self.initialize_ncn(
            &Config::find_program_address(&jito_restaking_program::id()).0,
            &ncn_pubkey,
            &ncn_admin,
            &ncn_base,
        )
        .await?;

        Ok(NcnRoot {
            ncn_pubkey,
            ncn_admin,
        })
    }

    #[allow(dead_code)]
    pub async fn do_initialize_ncn_vault_ticket(
        &mut self,
        ncn_root: &NcnRoot,
        vault: &Pubkey,
    ) -> Result<(), TestError> {
        let ncn_vault_ticket = NcnVaultTicket::find_program_address(
            &jito_restaking_program::id(),
            &ncn_root.ncn_pubkey,
            vault,
        )
        .0;

        self.initialize_ncn_vault_ticket(
            &Config::find_program_address(&jito_restaking_program::id()).0,
            &ncn_root.ncn_pubkey,
            vault,
            &ncn_vault_ticket,
            &ncn_root.ncn_admin,
            &self.payer.insecure_clone(),
        )
        .await
    }

    #[allow(dead_code)]
    pub async fn do_warmup_ncn_vault_ticket(
        &mut self,
        ncn_root: &NcnRoot,
        vault: &Pubkey,
    ) -> Result<(), TestError> {
        let ncn_vault_ticket = NcnVaultTicket::find_program_address(
            &jito_restaking_program::id(),
            &ncn_root.ncn_pubkey,
            vault,
        )
        .0;
        self.warmup_ncn_vault_ticket(
            &Config::find_program_address(&jito_restaking_program::id()).0,
            &ncn_root.ncn_pubkey,
            vault,
            &ncn_vault_ticket,
            &ncn_root.ncn_admin,
        )
        .await
    }

    pub async fn warmup_ncn_vault_ticket(
        &mut self,
        config: &Pubkey,
        ncn: &Pubkey,
        vault: &Pubkey,
        ncn_vault_ticket: &Pubkey,
        admin: &Keypair,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[warmup_ncn_vault_ticket(
                &jito_restaking_program::id(),
                config,
                ncn,
                vault,
                ncn_vault_ticket,
                &admin.pubkey(),
            )],
            Some(&self.payer.pubkey()),
            &[admin, &self.payer],
            blockhash,
        ))
        .await
    }

    #[allow(dead_code)]
    pub async fn do_cooldown_ncn_vault_ticket(
        &mut self,
        ncn_root: &NcnRoot,
        vault: &Pubkey,
    ) -> Result<(), TestError> {
        let ncn_vault_ticket = NcnVaultTicket::find_program_address(
            &jito_restaking_program::id(),
            &ncn_root.ncn_pubkey,
            vault,
        )
        .0;
        self.cooldown_ncn_vault_ticket(
            &Config::find_program_address(&jito_restaking_program::id()).0,
            &ncn_root.ncn_pubkey,
            vault,
            &ncn_vault_ticket,
            &ncn_root.ncn_admin,
        )
        .await
    }

    #[allow(dead_code)]
    pub async fn cooldown_ncn_vault_ticket(
        &mut self,
        config: &Pubkey,
        ncn: &Pubkey,
        vault: &Pubkey,
        ncn_vault_ticket: &Pubkey,
        admin: &Keypair,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[cooldown_ncn_vault_ticket(
                &jito_restaking_program::id(),
                config,
                ncn,
                vault,
                ncn_vault_ticket,
                &admin.pubkey(),
            )],
            Some(&self.payer.pubkey()),
            &[admin, &self.payer],
            blockhash,
        ))
        .await
    }

    #[allow(dead_code)]
    pub async fn do_ncn_warmup_operator(
        &mut self,
        ncn_root: &NcnRoot,
        operator_pubkey: &Pubkey,
    ) -> Result<(), TestError> {
        self.ncn_warmup_operator(
            &Config::find_program_address(&jito_restaking_program::id()).0,
            &ncn_root.ncn_pubkey,
            operator_pubkey,
            &NcnOperatorState::find_program_address(
                &jito_restaking_program::id(),
                &ncn_root.ncn_pubkey,
                operator_pubkey,
            )
            .0,
            &ncn_root.ncn_admin,
        )
        .await
    }

    #[allow(dead_code)]
    pub async fn do_ncn_cooldown_operator(
        &mut self,
        ncn_root: &NcnRoot,
        operator_pubkey: &Pubkey,
    ) -> Result<(), TestError> {
        self.ncn_cooldown_operator(
            &Config::find_program_address(&jito_restaking_program::id()).0,
            &ncn_root.ncn_pubkey,
            operator_pubkey,
            &NcnOperatorState::find_program_address(
                &jito_restaking_program::id(),
                &ncn_root.ncn_pubkey,
                operator_pubkey,
            )
            .0,
            &ncn_root.ncn_admin,
        )
        .await
    }

    #[allow(dead_code)]
    pub async fn ncn_cooldown_operator(
        &mut self,
        config: &Pubkey,
        ncn: &Pubkey,
        operator_pubkey: &Pubkey,
        ncn_operator_state: &Pubkey,
        admin: &Keypair,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[ncn_cooldown_operator(
                &jito_restaking_program::id(),
                config,
                ncn,
                operator_pubkey,
                ncn_operator_state,
                &admin.pubkey(),
            )],
            Some(&self.payer.pubkey()),
            &[admin, &self.payer],
            blockhash,
        ))
        .await
    }

    pub async fn ncn_warmup_operator(
        &mut self,
        config: &Pubkey,
        ncn: &Pubkey,
        operator_pubkey: &Pubkey,
        ncn_operator_state: &Pubkey,
        admin: &Keypair,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[ncn_warmup_operator(
                &jito_restaking_program::id(),
                config,
                ncn,
                operator_pubkey,
                ncn_operator_state,
                &admin.pubkey(),
            )],
            Some(&self.payer.pubkey()),
            &[admin, &self.payer],
            blockhash,
        ))
        .await
    }

    #[allow(dead_code)]
    pub async fn do_operator_warmup_ncn(
        &mut self,
        operator_root: &OperatorRoot,
        ncn_pubkey: &Pubkey,
    ) -> Result<(), TestError> {
        self.operator_warmup_ncn(
            &Config::find_program_address(&jito_restaking_program::id()).0,
            ncn_pubkey,
            &operator_root.operator_pubkey,
            &NcnOperatorState::find_program_address(
                &jito_restaking_program::id(),
                ncn_pubkey,
                &operator_root.operator_pubkey,
            )
            .0,
            &operator_root.operator_admin,
        )
        .await
    }

    #[allow(dead_code)]
    pub async fn operator_warmup_ncn(
        &mut self,
        config: &Pubkey,
        ncn_pubkey: &Pubkey,
        operator_pubkey: &Pubkey,
        ncn_operator_state: &Pubkey,
        admin: &Keypair,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[operator_warmup_ncn(
                &jito_restaking_program::id(),
                config,
                ncn_pubkey,
                operator_pubkey,
                ncn_operator_state,
                &admin.pubkey(),
            )],
            Some(&self.payer.pubkey()),
            &[admin, &self.payer],
            blockhash,
        ))
        .await
    }

    #[allow(dead_code)]
    pub async fn do_operator_cooldown_ncn(
        &mut self,
        operator_root: &OperatorRoot,
        ncn_pubkey: &Pubkey,
    ) -> Result<(), TestError> {
        self.operator_cooldown_ncn(
            &Config::find_program_address(&jito_restaking_program::id()).0,
            ncn_pubkey,
            &operator_root.operator_pubkey,
            &NcnOperatorState::find_program_address(
                &jito_restaking_program::id(),
                ncn_pubkey,
                &operator_root.operator_pubkey,
            )
            .0,
            &operator_root.operator_admin,
        )
        .await
    }

    pub async fn operator_cooldown_ncn(
        &mut self,
        config: &Pubkey,
        ncn_pubkey: &Pubkey,
        operator_pubkey: &Pubkey,
        ncn_operator_state: &Pubkey,
        admin: &Keypair,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[operator_cooldown_ncn(
                &jito_restaking_program::id(),
                config,
                ncn_pubkey,
                operator_pubkey,
                ncn_operator_state,
                &admin.pubkey(),
            )],
            Some(&self.payer.pubkey()),
            &[admin, &self.payer],
            blockhash,
        ))
        .await
    }

    #[allow(dead_code)]
    pub async fn do_initialize_ncn_operator_state(
        &mut self,
        ncn_root: &NcnRoot,
        operator: &Pubkey,
    ) -> Result<(), TestError> {
        let ncn_operator_state = NcnOperatorState::find_program_address(
            &jito_restaking_program::id(),
            &ncn_root.ncn_pubkey,
            operator,
        )
        .0;

        self.initialize_ncn_operator_state(
            &Config::find_program_address(&jito_restaking_program::id()).0,
            &ncn_root.ncn_pubkey,
            operator,
            &ncn_operator_state,
            &ncn_root.ncn_admin,
            &self.payer.insecure_clone(),
        )
        .await
    }

    #[allow(dead_code)]
    pub async fn do_initialize_ncn_vault_slasher_ticket(
        &mut self,
        ncn_root: &NcnRoot,
        vault: &Pubkey,
        slasher: &Pubkey,
        max_slash_amount: u64,
    ) -> Result<(), TestError> {
        let ncn_vault_ticket = NcnVaultTicket::find_program_address(
            &jito_restaking_program::id(),
            &ncn_root.ncn_pubkey,
            vault,
        )
        .0;
        let ncn_slasher_ticket = NcnVaultSlasherTicket::find_program_address(
            &jito_restaking_program::id(),
            &ncn_root.ncn_pubkey,
            vault,
            slasher,
        )
        .0;

        self.initialize_ncn_vault_slasher_ticket(
            &Config::find_program_address(&jito_restaking_program::id()).0,
            &ncn_root.ncn_pubkey,
            vault,
            slasher,
            &ncn_vault_ticket,
            &ncn_slasher_ticket,
            &ncn_root.ncn_admin,
            &self.payer.insecure_clone(),
            max_slash_amount,
        )
        .await
    }

    #[allow(dead_code)]
    pub async fn do_warmup_ncn_vault_slasher_ticket(
        &mut self,
        ncn_root: &NcnRoot,
        vault: &Pubkey,
        slasher: &Pubkey,
    ) -> Result<(), TestError> {
        let ncn_vault_ticket = NcnVaultTicket::find_program_address(
            &jito_restaking_program::id(),
            &ncn_root.ncn_pubkey,
            vault,
        )
        .0;
        let ncn_slasher_ticket = NcnVaultSlasherTicket::find_program_address(
            &jito_restaking_program::id(),
            &ncn_root.ncn_pubkey,
            vault,
            slasher,
        )
        .0;

        self.warmup_ncn_vault_slasher_ticket(
            &Config::find_program_address(&jito_restaking_program::id()).0,
            &ncn_root.ncn_pubkey,
            vault,
            slasher,
            &ncn_vault_ticket,
            &ncn_slasher_ticket,
            &ncn_root.ncn_admin,
        )
        .await
    }

    pub async fn warmup_ncn_vault_slasher_ticket(
        &mut self,
        config: &Pubkey,
        ncn: &Pubkey,
        vault: &Pubkey,
        slasher: &Pubkey,
        ncn_vault_ticket: &Pubkey,
        ncn_slasher_ticket: &Pubkey,
        admin: &Keypair,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[warmup_ncn_vault_slasher_ticket(
                &jito_restaking_program::id(),
                config,
                ncn,
                vault,
                slasher,
                ncn_vault_ticket,
                ncn_slasher_ticket,
                &admin.pubkey(),
            )],
            Some(&self.payer.pubkey()),
            &[admin, &self.payer],
            blockhash,
        ))
        .await
    }

    pub async fn initialize_ncn(
        &mut self,
        config: &Pubkey,
        ncn: &Pubkey,
        ncn_admin: &Keypair,
        ncn_base: &Keypair,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[initialize_ncn(
                &jito_restaking_program::id(),
                config,
                ncn,
                &ncn_admin.pubkey(),
                &ncn_base.pubkey(),
            )],
            Some(&ncn_admin.pubkey()),
            &[&ncn_admin, &ncn_base],
            blockhash,
        ))
        .await
    }

    pub async fn initialize_ncn_vault_ticket(
        &mut self,
        config: &Pubkey,
        ncn: &Pubkey,
        vault: &Pubkey,
        ncn_vault_ticket: &Pubkey,
        ncn_admin: &Keypair,
        payer: &Keypair,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[initialize_ncn_vault_ticket(
                &jito_restaking_program::id(),
                config,
                ncn,
                vault,
                ncn_vault_ticket,
                &ncn_admin.pubkey(),
                &payer.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[ncn_admin, payer],
            blockhash,
        ))
        .await
    }

    pub async fn initialize_ncn_operator_state(
        &mut self,
        config: &Pubkey,
        ncn: &Pubkey,
        operator: &Pubkey,
        ncn_operator_state: &Pubkey,
        ncn_admin: &Keypair,
        payer: &Keypair,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[initialize_ncn_operator_state(
                &jito_restaking_program::id(),
                config,
                ncn,
                operator,
                ncn_operator_state,
                &ncn_admin.pubkey(),
                &payer.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[ncn_admin, payer],
            blockhash,
        ))
        .await
    }

    pub async fn initialize_ncn_vault_slasher_ticket(
        &mut self,
        config: &Pubkey,
        ncn: &Pubkey,
        vault: &Pubkey,
        slasher: &Pubkey,
        ncn_vault_ticket: &Pubkey,
        ncn_slasher_ticket: &Pubkey,
        ncn_admin: &Keypair,
        payer: &Keypair,
        max_slash_amount: u64,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[initialize_ncn_vault_slasher_ticket(
                &jito_restaking_program::id(),
                config,
                ncn,
                vault,
                slasher,
                ncn_vault_ticket,
                ncn_slasher_ticket,
                &ncn_admin.pubkey(),
                &payer.pubkey(),
                max_slash_amount,
            )],
            Some(&payer.pubkey()),
            &[ncn_admin, payer],
            blockhash,
        ))
        .await
    }

    #[allow(dead_code)]
    pub async fn ncn_set_admin(
        &mut self,
        ncn: &Pubkey,
        old_admin: &Keypair,
        new_admin: &Keypair,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[ncn_set_admin(
                &jito_restaking_program::id(),
                ncn,
                &old_admin.pubkey(),
                &new_admin.pubkey(),
            )],
            Some(&old_admin.pubkey()),
            &[old_admin, new_admin],
            blockhash,
        ))
        .await
    }

    #[allow(dead_code)]
    pub async fn operator_set_admin(
        &mut self,
        operator: &Pubkey,
        old_admin: &Keypair,
        new_admin: &Keypair,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[operator_set_admin(
                &jito_restaking_program::id(),
                operator,
                &old_admin.pubkey(),
                &new_admin.pubkey(),
            )],
            Some(&old_admin.pubkey()),
            &[old_admin, new_admin],
            blockhash,
        ))
        .await
    }

    #[allow(dead_code)]
    pub async fn operator_set_secondary_admin(
        &mut self,
        operator: &Pubkey,
        old_admin: &Keypair,
        new_admin: &Keypair,
        operator_admin_role: OperatorAdminRole,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[operator_set_secondary_admin(
                &jito_restaking_program::id(),
                operator,
                &old_admin.pubkey(),
                &new_admin.pubkey(),
                operator_admin_role,
            )],
            Some(&old_admin.pubkey()),
            &[old_admin],
            blockhash,
        ))
        .await
    }

    pub async fn initialize_operator(
        &mut self,
        config: &Pubkey,
        operator: &Pubkey,
        admin: &Keypair,
        base: &Keypair,
        operator_fee_bps: u16,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[initialize_operator(
                &jito_restaking_program::id(),
                config,
                operator,
                &admin.pubkey(),
                &base.pubkey(),
                operator_fee_bps,
            )],
            Some(&admin.pubkey()),
            &[admin, base],
            blockhash,
        ))
        .await
    }

    pub async fn initialize_operator_vault_ticket(
        &mut self,
        config: &Pubkey,
        operator: &Pubkey,
        vault: &Pubkey,
        operator_vault_ticket: &Pubkey,
        admin: &Keypair,
        payer: &Keypair,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[initialize_operator_vault_ticket(
                &jito_restaking_program::id(),
                config,
                operator,
                vault,
                operator_vault_ticket,
                &admin.pubkey(),
                &payer.pubkey(),
            )],
            Some(&payer.pubkey()),
            &[admin, payer],
            blockhash,
        ))
        .await
    }

    #[allow(dead_code)]
    pub async fn operator_set_fee(
        &mut self,
        config: &Pubkey,
        operator: &Pubkey,
        admin: &Keypair,
        new_fee_bps: u16,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;

        self.process_transaction(&Transaction::new_signed_with_payer(
            &[operator_set_fee(
                &jito_restaking_program::id(),
                config,
                operator,
                &admin.pubkey(),
                new_fee_bps,
            )],
            Some(&self.payer.pubkey()),
            &[admin, &self.payer],
            blockhash,
        ))
        .await
    }

    #[allow(dead_code)]
    pub async fn ncn_delegate_token_account(
        &mut self,
        ncn_pubkey: &Pubkey,
        delegate_admin: &Keypair,
        token_mint: &Pubkey,
        token_account: &Pubkey,
        delegate: &Pubkey,
        token_program_id: &Pubkey,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self.process_transaction(&Transaction::new_signed_with_payer(
            &[jito_restaking_sdk::sdk::ncn_delegate_token_account(
                &jito_restaking_program::id(),
                ncn_pubkey,
                &delegate_admin.pubkey(),
                token_mint,
                token_account,
                delegate,
                token_program_id,
            )],
            Some(&self.payer.pubkey()),
            &[&self.payer, delegate_admin],
            blockhash,
        ))
        .await
    }

    #[allow(dead_code)]
    pub async fn operator_delegate_token_account(
        &mut self,
        operator_pubkey: &Pubkey,
        delegate_admin: &Keypair,
        token_mint: &Pubkey,
        token_account: &Pubkey,
        delegate: &Pubkey,
        token_program_id: &Pubkey,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self.process_transaction(&Transaction::new_signed_with_payer(
            &[jito_restaking_sdk::sdk::operator_delegate_token_account(
                &jito_restaking_program::id(),
                operator_pubkey,
                &delegate_admin.pubkey(),
                token_mint,
                token_account,
                delegate,
                token_program_id,
            )],
            Some(&self.payer.pubkey()),
            &[&self.payer, delegate_admin],
            blockhash,
        ))
        .await
    }

    pub async fn process_transaction(&mut self, tx: &Transaction) -> Result<(), TestError> {
        self.banks_client
            .process_transaction_with_preflight_and_commitment(
                tx.clone(),
                CommitmentLevel::Processed,
            )
            .await?;
        Ok(())
    }

    pub async fn _airdrop(&mut self, to: &Pubkey, sol: f64) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self.banks_client
            .process_transaction_with_preflight_and_commitment(
                Transaction::new_signed_with_payer(
                    &[transfer(
                        &self.payer.pubkey(),
                        to,
                        sol_str_to_lamports(&sol.to_string()).unwrap(),
                    )],
                    Some(&self.payer.pubkey()),
                    &[&self.payer],
                    blockhash,
                ),
                CommitmentLevel::Processed,
            )
            .await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn set_config_admin(
        &mut self,
        config: &Pubkey,
        old_admin: &Keypair,
        new_admin: &Keypair,
    ) -> Result<(), TestError> {
        let blockhash = self.banks_client.get_latest_blockhash().await?;
        self.process_transaction(&Transaction::new_signed_with_payer(
            &[set_config_admin(
                &jito_restaking_program::id(),
                config,
                &old_admin.pubkey(),
                &new_admin.pubkey(),
            )],
            Some(&old_admin.pubkey()),
            &[old_admin],
            blockhash,
        ))
        .await
    }
}

// #[track_caller]
// #[inline(always)]
// pub fn assert_restaking_error<T>(
//     test_error: Result<T, TestError>,
//     restaking_error: RestakingError,
// ) {
//     assert!(test_error.is_err());
//     assert_eq!(
//         test_error.err().unwrap().to_transaction_error().unwrap(),
//         TransactionError::InstructionError(0, InstructionError::Custom(restaking_error as u32))
//     );
// }
