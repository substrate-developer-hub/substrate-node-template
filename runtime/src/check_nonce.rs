use codec::{Decode, Encode};
use frame_support::dispatch::{DispatchInfo, Pays};
use frame_system::Config;
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{DispatchInfoOf, Dispatchable, One, SignedExtension, Zero},
    transaction_validity::{
        InvalidTransaction, TransactionLongevity, TransactionValidity, TransactionValidityError,
        ValidTransaction,
    },
    Saturating,
};
use sp_std::vec;
use subtensor_macros::freeze_struct;

/// Nonce check and increment to give replay protection for transactions.
///
/// # Transaction Validity
///
/// This extension affects `requires` and `provides` tags of validity, but DOES NOT
/// set the `priority` field. Make sure that AT LEAST one of the signed extension sets
/// some kind of priority upon validating transactions.
#[freeze_struct("610b76f62cdb521e")]
#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct CheckNonce<T: Config>(#[codec(compact)] pub T::Nonce);

impl<T: Config> CheckNonce<T> {
    /// utility constructor. Used only in client/factory code.
    pub fn from(nonce: T::Nonce) -> Self {
        Self(nonce)
    }
}

impl<T: Config> sp_std::fmt::Debug for CheckNonce<T> {
    #[cfg(feature = "std")]
    fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        write!(f, "CheckNonce({})", self.0)
    }

    #[cfg(not(feature = "std"))]
    fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        Ok(())
    }
}

impl<T: Config> SignedExtension for CheckNonce<T>
where
    T::RuntimeCall: Dispatchable<Info = DispatchInfo>,
{
    type AccountId = T::AccountId;
    type Call = T::RuntimeCall;
    type AdditionalSigned = ();
    type Pre = ();
    const IDENTIFIER: &'static str = "CheckNonce";

    fn additional_signed(&self) -> sp_std::result::Result<(), TransactionValidityError> {
        Ok(())
    }

    fn pre_dispatch(
        self,
        who: &Self::AccountId,
        _call: &Self::Call,
        info: &DispatchInfoOf<Self::Call>,
        _len: usize,
    ) -> Result<(), TransactionValidityError> {
        let mut account = frame_system::Account::<T>::get(who);
        match info.pays_fee {
            Pays::Yes => {
                if account.providers.is_zero() && account.sufficients.is_zero() {
                    // Nonce storage not paid for
                    return Err(InvalidTransaction::Payment.into());
                }
            }
            // not check providers and sufficients for Pays::No extrinsic
            Pays::No => {}
        }

        if self.0 != account.nonce {
            return Err(if self.0 < account.nonce {
                InvalidTransaction::Stale
            } else {
                InvalidTransaction::Future
            }
            .into());
        }
        account.nonce.saturating_inc();
        frame_system::Account::<T>::insert(who, account);
        Ok(())
    }

    fn validate(
        &self,
        who: &Self::AccountId,
        _call: &Self::Call,
        info: &DispatchInfoOf<Self::Call>,
        _len: usize,
    ) -> TransactionValidity {
        let account = frame_system::Account::<T>::get(who);
        match info.pays_fee {
            Pays::Yes => {
                if account.providers.is_zero() && account.sufficients.is_zero() {
                    // Nonce storage not paid for
                    return Err(InvalidTransaction::Payment.into());
                }
            }
            // not check providers and sufficients for Pays::No extrinsic
            Pays::No => {}
        }
        if self.0 < account.nonce {
            return InvalidTransaction::Stale.into();
        }

        let provides = vec![Encode::encode(&(who, self.0))];
        let requires = if account.nonce < self.0 {
            vec![Encode::encode(&(who, self.0.saturating_sub(One::one())))]
        } else {
            vec![]
        };

        Ok(ValidTransaction {
            priority: 0,
            requires,
            provides,
            longevity: TransactionLongevity::MAX,
            propagate: true,
        })
    }
}
