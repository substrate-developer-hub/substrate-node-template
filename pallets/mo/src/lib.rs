#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit="256"]
use core::str;
use core::str::FromStr;
/// Module to manage the function for the MarketPlace
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch, ensure, traits::Currency,codec::Decode,
};
use frame_system::{ensure_root, ensure_signed};
use sp_std::prelude::*;


#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// Module configuration
pub trait Config: frame_system::Config {
    //pub trait Config: frame_system::Config + Sized {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type Currency: Currency<Self::AccountId>;
}
pub type Balance = u128;

// The runtime storage items
decl_storage! {
    trait Store for Module<T: Config> as marketplace {
        // we use a safe crypto hashing by blake2_128
        // Seller data storage
        Sellers get(fn get_seller): map hasher(blake2_128_concat) T::AccountId => Option<Vec<u8>>;
        // Product Departments
        ProductDepartments get(fn get_products_department): map hasher(blake2_128_concat) u32 => Option<Vec<u8>>;
        // Product Categories
        ProductCategories get(fn get_products_category): double_map hasher(blake2_128_concat) u32,hasher(blake2_128_concat) u32 => Option<Vec<u8>>;
        // Product Colors
        ProductColors get(fn get_products_color): map hasher(blake2_128_concat) u32 => Option<Vec<u8>>;
        // Product Sizes
        ProductSizes get(fn get_products_size): map hasher(blake2_128_concat) u32 => Option<Vec<u8>>;
        // Products Data
        Products get(fn get_product): map hasher(blake2_128_concat) u32 => Option<Vec<u8>>;
        // Standard Iso country code and official name
        IsoCountries get(fn get_iso_country): map hasher(blake2_128_concat) Vec<u8> => Option<Vec<u8>>;
        // Standard Iso dial code for country code
        IsoDialcode get(fn get_iso_dialcode): map hasher(blake2_128_concat) Vec<u8> => Option<Vec<u8>>;
        // Currencies data
        Currencies get(fn get_currency): map hasher(blake2_128_concat) Vec<u8> => Option<Vec<u8>>;
        // Manufacturers name and website
        Manufacturers get(fn get_manufacturer): map hasher(blake2_128_concat) u32 => Option<Vec<u8>>;
        // Brand name and Manufacturer
        Brands get(fn get_brand): map hasher(blake2_128_concat) u32 => Option<Vec<u8>>;
        // Product Models
        ProductModels get(fn get_product_model): map hasher(blake2_128_concat) u32 => Option<Vec<u8>>;
        // Shipping Companies
        Shippers get(fn get_shipper): map hasher(blake2_128_concat) u32 => Option<Vec<u8>>;
        // Shipping Companies
        ShippingRates get(fn get_shipper_rate): map hasher(blake2_128_concat) u32 => Option<Vec<u8>>;
        // Login data: email and password hashes. The password is hashed with ARGON2 and then furher encrypted with 3 layers of symmetric encryption.
        LoginData get(fn get_login_data): map hasher(blake2_128_concat) Vec<u8> => Option<Vec<u8>>;
        // Email -> Account id
        EmailAccount get(fn get_email_account): map hasher(blake2_128_concat) Vec<u8> => T::AccountId;
        // Encrypted Secret Seed (double encrption)
        EmailEncryptedSeed get(fn get_encrypted_seed): map hasher(blake2_128_concat) Vec<u8> => Option<Vec<u8>>;
    }
}

// We generate events to inform the users of succesfully actions.
decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
    {
        MarketPlaceDepartmentCreated(u32, Vec<u8>),         // New department created
        MarketPlaceDepartmentDestroyed(u32),                // Department has been destroyed/removed
        MarketPlaceSellerCreated(AccountId, Vec<u8>),       // New seller has been created
        MarketPlaceSellerDestroyed(AccountId),              // Seller destroyed
        MarketPlaceCategoryCreated(u32, u32, Vec<u8>),      // New producct category has been created
        MarketPlaceCategoryDestroyed(u32, u32),             // Product category has been destroyed
        MarketPlaceIsoCountryCreated(Vec<u8>, Vec<u8>),     // New Iso contry code has been created
        MarketPlaceIsoCountryDestroyed(Vec<u8>),            // Iso contry code has been destroyed
        MarketPlaceIsoDialCodeCreated(Vec<u8>, Vec<u8>),    // New country dial code has been created
        MarketPlaceIsoDialCodeDestroyed(Vec<u8>),           // A country dial code has been destroyed
        MarketPlaceCurrencyCodeCreated(Vec<u8>, Vec<u8>),   // A new currency has been created
        MarketPlaceCurrencyDestroyed(Vec<u8>),              // A currency has been destroyed
        MarketPlaceColorCreated(u32,Vec<u8>),               // A new color has been created
        MarketPlaceColorDestroyed(u32),                     // A color has been removed
        MarketPlaceSizeCreated(u32,Vec<u8>),                // A new size table has been created
        MarketPlaceSizeDestroyed(u32),                      // A size table has been removed
        MarketPlaceManufacturerCreated(u32,Vec<u8>),        // A new manufacturer has been created
        MarketPlaceManufacturerDestroyed(u32),              // A manufacturer has been removed
        MarketPlaceBrandCreated(u32,Vec<u8>),               // A new brand has been created
        MarketPlaceBrandDestroyed(u32),                     // A brand has been removed
        MarketPlaceProductModelCreated(u32,Vec<u8>),        // a new product model has been created
        MarketPlaceProductModelDestroyed(u32),              // a product model has been removed
        MarketPlaceShipperCreated(u32,Vec<u8>),             // A new shipper has been created
        MarketPlaceShipperDestroyed(u32),                   // A shipper has been removed
        MarketShippingRateCreated(u32,Vec<u8>),             // A new shipping rate has been created
        MarketShippingRateDestroyed(u32),                   // A shipping rate has been removed
        MarketPlaceProductUpdated(u32,Vec<u8>),             // A product has been created or updated
        MarketPlaceLoginDataCreated(Vec<u8>, Vec<u8>,AccountId), // A new login data has been created
        MarketPlaceLoginDataDestroyed(Vec<u8>),             // A login data has been destroyed
        MarketPlaceLoginPwdChanged(Vec<u8>, Vec<u8>),       // password changed
    }
);

// Errors to inform users that something went wrong.
decl_error! {
    pub enum Error for Module<T: Config> {
        /// Uid cannot be zero
        UidCannotBeZero,
        /// Configuration data is too short
        ConfigurationTooShort,
        /// Configuration data is too long
        ConfigurationTooLong,
        /// Seller is already present
        SellerAlreadyPresent,
        /// Invalid json sintax
        InvalidJson,
        /// Department Description is too short, it should be > 3 bytes
        DepartmentDescriptionTooShort,
        // Department Description is too long, it should be < 128 bytes
        DepartmentDescriptionTooLong,
        /// Department Id cannot be equale to zero
        DepartmentUidCannotBeZero,
        /// Department is already present on chain
        DepartmentAlreadyPresent,
        /// Department not found on chain
        DepartmentNotFound,
        /// Category ID cannot be equal to zero
        CategoryUidCannotBeZero,
        /// Category Description is too short
        CategoryDescriptionTooShort,
        /// Category Description is too long
        CategoryDescriptionTooLong,
        /// Category has not been found
        CategoryNotFound,
        /// Product category is already present on chain
        ProductCategoryAlreadyPresent,
        /// Product category not found on chain
        ProductCategoryNotFound,
        /// The country code is wrong, it must be long 2 bytes
        WrongLengthCountryCode,
        /// The country name is too short, it must be >=3
        CountryNameTooShort,
        /// Country code already present on chain
        CountryCodeAlreadyPresent,
        /// Country code not found on chain
        CountryCodeNotFound,
        /// International Dial code is too short it must be at the least 2 bytes
        DialcodeTooShort,
        /// Seller type can be 1 for Company, 2 for Professional, 3 for Private
        SellerTypeInvalid,
        /// Seller name is too short, it must be at least 5 bytes
        SellerNameTooShort,
        /// The Sellet city is too short, it mut be at the least 5 bytes
        SellerCityTooShort,
         /// The Sellet city is too short, it mut be at the least 5 bytes
         SellerCityTooLong,
        /// The seller address is too long, maximum 128 bytes
        SellerAddressTooLong,
        /// The seller zip code is too long, maximum 12 bytes
        SellerZipCodeTooLong,
        /// Po Box is too long, maximum 64 bytes
        SellerPoBoxTooLong,
        /// Seller certification description is too short, must be > 3 bytes
        SellerCertificationDescriptionTooShort,
        /// Seller certification description is too long, must be <= 64 bytes
        SellerCertificationDescriptionTooLong,
        /// Seller certificate verification is too short
        SellerCertificateVerificationTooShort,
        /// Seller certificate verification is too long
        SellerCertificateVerificationTooLong,
        /// Seller info email is wrong
        SellerInfoEmailIsWrong,
        /// Seller support email is wrong
        SellerSupportEmailIsWrong,
        /// Phone description is too short, it should be at the least 4 bytes
        SellerPhoneDescriptionTooShort,
        /// Phone description is too long, maximum 64 bytes
        SellerPhoneDescriptionTooLong,
        /// Phone number is too short at the least > 3 bytes
        SellerPhoneNumberTooShort,
        /// Phone number is too long, maximum 21 bytes
        SellerPhoneNumberTooLong,
        /// Categories of product/service sold from seller is missing
        SellerCategoriesMissing,
        /// Included countries for shipment are missing at the least "countries":[], should be set
        SellercountriesMissing,
        /// the inout fiels is not set for the the country, it should be 0 for included, 1 for excluded country with default worldwide
        IncludedExcludedCountryValueIsMissing,
        /// The latitude of the center point for the shipment area, is missing
        ShipmentAreaCenterLatitudeIsMissing,
        /// The longitude of the center point for the shipment area, is missing
        ShipmentAreaCenterLongitudeIsMissing,
        /// The latitude of the border point for the shipment area, is missing
        ShipmentAreaBorderLatitudeIsMissing,
        /// The longitude of the border point for the shipment area, is missing
        ShipmentAreaBorderLongituteIsMissing,
        /// Seller Social Url is wrong
        SellerSocialUrlIsWrong,
        /// Seller web site is wrong
        SellerWebsiteUrlIsWrong,
        /// Seller the url for certificate verification is wrong
        SellerCertificationUrlIsWrong,
        /// Seller phone number is wrong
        SellerPhoneNumberIsWrong,
        /// Seller data has not been found on chain
        SellerDataNotFound,
        /// Seller Default language is wrong
        SellerDefaultLanguageIsWrong,
        /// Seller default unit of measurement is wrong
        SellerDefaultUnitMeasurementIsWrong,
        /// Default return policy in days cannot be more than 10 years
        DefaultReturnPolicyIsExcessive,
        /// Currency code should be between 2 and 5 bytes
        WrongLengthCurrencyCode,
        /// Currency name is too short, at the last 3 bytes are required 
        CurrencyNameTooShort,
        /// Currency name is too long, maximum allowed are 32 bytes
        CurrencyNameTooLong,
        /// Currency category can be "c" for crypto currency like Bitcoin or "f" for fiat/national currency like USD
        CurrencyCategoryIswrong,
        /// Blockchain name is too short, minimum 3 bytes
        BlockchainNameTooShort,
        /// Blockchain name is too long, maximum 32 bytes
        BlockchainNameTooLong,
        /// Currency code is already present
        CurrencyCodeAlreadyPresent,
        /// Currency code has not been found
        CurrencyCodeNotFound,
        /// Product Description is too short, minimum 10 bytes
        ProductDescriptionTooShort,
        /// Product Description is too long, maximum 64 bytes
        ProductDescriptionTooLong,
        /// Product Long Description is too short, minimum 64 bytes
        ProductLongDescriptionTooShort,
        /// Product Long Description is too long, maximum 4096 bytes
        ProductLongDescriptionTooLong,
        /// Product price must be > zero
        ProductPriceCannotBeZero,
        /// Specification must be >0 and < 8192
        SpecificationsIsdInvalid,
        /// Media files cannot be empty, high quality description is required.
        MediaCannotBeEmpty,
        /// Media Description is wrong, cannot be empty
        MediaDescriptionIswrong,
        /// Media filename is wrong, cannot be empty
        MediaFileNameIsWrong,
        /// Media Ipfs Address is wrong
        MediaIpfsAddressIsWrong,
        /// Color Uid cannot be zero
        ColorUidCannotBeZero,
        /// Color Description is too short
        ColorDescriptionTooShort,
        /// Color Description is too long
        ColorDescriptionTooLong,
        /// Color already present with the same uid
        ColorAlreadyPresent,
        /// Color not found
        ColorNotFound,
        /// Size Uid cannot be Zero
        SizeUidCannotBeZero,
        /// Size info cannot be > 8192 bytes
        SizeInfoTooLong,
        /// Size already present with the same UID
        SizeAlreadyPresent,
        /// Size has not been found
        SizeNotFound,
        /// Size code is missing
        SizeCodeIsMissing,
        /// Size description is missing
        SizeDescriptionIsMissing,
        /// Size area is missing
        SizeAreaIsMissing,
        /// Manufacturer Id cannot be empty
        ManufacturerUidCannotBeZero,
        /// Manufacturer name must be minimum 4 bytes
        ManufacturerNameIsTooShort,
        /// Manufacturer name can be maximum 64 bytes
        ManufacturerNameIsTooLong,
        /// Manufacturer website must be minimum 4 bytes
        ManufacturerWebsiteIsTooShort,
        /// Manufacturer name can be maximum 128 bytes
        ManufacturerWebsiteIsTooLong,
        /// Manufacturer is already present
        ManufacturerAlreadyPresent,
        /// Manufacturer has not been found
        ManufacturerNotFound,
        /// Brand Id cannot be empty
        BrandUidCannotBeZero,
        /// Brand name must be minimum 4 bytes
        BrandNameIsTooShort,
        /// Brand name can be maximum 64 bytes
        BrandNameIsTooLong,
        /// Manufacturer is already present
        BrandAlreadyPresent,
        /// Brand has not been found
        BrandNotFound,
        /// Model id cannot be empty or zero
        ModelUidCannotBeZero,
        /// Model name must be minimum 3 bytes
        ModelNameIsTooShort,
        /// Model name must be maximum 32 bytes
        ModelNameIsTooLong,
        /// Model is already present 
        ModelAlreadyPresent,
        /// Model has not been found
        ModelNotfound,
        /// Shipper Id cannot be zero/empty
        ShipperUidCannotBeZero,
        /// Shipper name must be longer than 3 bytes
        ShipperNameIsTooShort,
        /// Shipper name cannot be longer than 64 bytes
        ShipperNameIsTooLong,
        /// Shipper website must be longer than 4 bytes
        ShipperWebsiteIsTooShort,
        /// Shipper website cannot be longer than 64 bytes
        ShipperWebsiteIsTooLong,
        /// Shipper is already present
        ShipperAlreadyPresent,
        /// Country of origin is not present on chain
        OriginCountryNotPresent,
        /// Destination country is not present on chain
        DestinationCountryNotPresent,
        /// Json field is too long
        JsonIsTooLong,
        /// Shipper has not been found
        ShipperNotFound,
        /// Shipping rate ID cannot be zero/empty
        ShippingRateUidCannotBeZero,
        /// Shipper id is missing
        ShipperIdIsMissing,
        /// From kg field is missing
        FromKgIsMissing,
        /// To kg field is missing
        ToKgIsMissing,
        /// Shipping rate cannot be zero
        ShippingRateCannotbeZero,
        /// Shipping rate cannot be found
        ShippingRatesNotFound,
        /// Dimension lenght size is wrong
        DimensionWrongLength,
        /// Dimension wide size is wrong
        DimensionWrongWide,
        /// Dimension height size is wrong
        DimensionWrongHeight,
        /// Dimension weight size is wrong
        DimensionWrongWeight,
        /// UPC code is missing or is wrong
        UniversalProductCodeIsWrong,
        /// Center Latitude of the area is missing
        CenterLatitudeIsMissing,
        /// Center Longitude of the area is missing
        CenterLongitudeIsMissing,
        /// Border Latitude of the area is missing
        BorderLatitudeIsMissing,
        /// Center Longitude of the area is missing
        BorderLongitudeIsMissing,
        /// Invalid Api Url, should be an https or http address
        InvalidApiUrl,
        /// Language code is wrong
        LanguageCodeIsWrong,
        /// Wrong lenght for the Encrypted Email
        WrongLengthEmailHash,
        /// Wrong lenght for the Encrypted Password
        WrongLengthEncryptedPassword,
        /// Email hash is already present on chain
        EmailHashAlreadyPresent,
        // Email hash has not been found on chain
        EmailHashNotFound,
        /// Signer of transaction is not authorized to execute it
        SignerIsNotAuthorized,  
    }
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        // Errors must be initialized
        type Error = Error<T>;
        // Events must be initialized
        fn deposit_event() = default;

        /// Create a new product department
        #[weight = 1000]
        pub fn create_product_department(origin, uid: u32, description: Vec<u8>) -> dispatch::DispatchResult {
            // check the request is signed from root
            let _sender = ensure_root(origin)?;
            // check uid >0
            ensure!(uid > 0, Error::<T>::DepartmentUidCannotBeZero);
            //check description length
            ensure!(description.len() > 3, Error::<T>::DepartmentDescriptionTooShort);
            ensure!(description.len() < 128, Error::<T>::DepartmentDescriptionTooLong);
            // check the department is not alreay present on chain
            ensure!(ProductDepartments::contains_key(uid)==false, Error::<T>::DepartmentAlreadyPresent);
            // store the department
            ProductDepartments::insert(uid,description.clone());
            // Generate event
            Self::deposit_event(RawEvent::MarketPlaceDepartmentCreated(uid,description));
            // Return a successful DispatchResult
            Ok(())
        }
        /// Destroy a product department
        #[weight = 1000]
        pub fn destroy_product_department(origin, uid: u32) -> dispatch::DispatchResult {
            // check the request is signed from Super User
            let _sender = ensure_root(origin)?;
            // verify the department exists
            ensure!(ProductDepartments::contains_key(&uid)==true, Error::<T>::DepartmentNotFound);
            // Remove department
            ProductDepartments::take(uid);
            // Generate event
            //it can leave orphans, anyway it's a decision of the super user
            Self::deposit_event(RawEvent::MarketPlaceDepartmentDestroyed(uid));
            // Return a successful DispatchResult
            Ok(())
        }
        /// Create a new product category
        #[weight = 1000]
        pub fn create_product_category(origin, uiddepartment: u32, uidcategory: u32, description: Vec<u8>) -> dispatch::DispatchResult {
            // check the request is signed from the Super User
            let _sender = ensure_root(origin)?;
            // check uid department >0
            ensure!(uiddepartment > 0, Error::<T>::DepartmentUidCannotBeZero);
            // check uid category >0
            ensure!(uidcategory > 0, Error::<T>::CategoryUidCannotBeZero);
            //check description length
            ensure!(description.len() > 3, Error::<T>::CategoryDescriptionTooShort);
            ensure!(description.len() < 128, Error::<T>::CategoryDescriptionTooLong);
            // check the department is  alreay present on chain
            ensure!(ProductDepartments::contains_key(uiddepartment)==true, Error::<T>::DepartmentNotFound);
            // check the department/category is not alreay present on chain
            ensure!(ProductCategories::contains_key(uiddepartment,uidcategory)==false, Error::<T>::ProductCategoryAlreadyPresent);
            // store the department
            ProductCategories::insert(uiddepartment,uidcategory,description.clone());
            // Generate event
            Self::deposit_event(RawEvent::MarketPlaceCategoryCreated(uiddepartment,uidcategory,description));
            // Return a successful DispatchResult
            Ok(())
        }
        /// Destroy a product category
        #[weight = 1000]
        pub fn destroy_product_category(origin, uiddepartment: u32, uidcategory: u32) -> dispatch::DispatchResult {
            // check the request is signed from the Super User
            let _sender = ensure_root(origin)?;
            // verify the department/category exists
            ensure!(ProductCategories::contains_key(&uiddepartment,&uidcategory)==true, Error::<T>::ProductCategoryNotFound);
            // Remove department
            ProductCategories::take(uiddepartment,uidcategory);
            // Generate event
            //it can leave orphans, anyway it's a decision of the super user
            Self::deposit_event(RawEvent::MarketPlaceCategoryDestroyed(uiddepartment, uidcategory));
            // Return a successful DispatchResult
            Ok(())
        }
        /// Create a new seller
        #[weight = 10000]
        pub fn create_update_seller(origin, configuration: Vec<u8>) -> dispatch::DispatchResult {
            // check the request is signed
            let mut sender = ensure_signed(origin)?;
            let originalsigner=sender.clone();
            //check configuration length
            ensure!(configuration.len() > 12, Error::<T>::ConfigurationTooShort);
            ensure!(configuration.len() < 8192, Error::<T>::ConfigurationTooLong);
            // check json validity
            ensure!(json_check_validity(configuration.clone()),Error::<T>::InvalidJson);
            // checking seller type 1= Company, 2= Freelancer, 3= Individual, 4 == Government Agency 4 == NGO
            let sellertype=json_get_value(configuration.clone(),"sellertype".as_bytes().to_vec());
            let sellertypeu32=vecu8_to_u32(sellertype);
            ensure!(sellertypeu32==1 || sellertypeu32==2 || sellertypeu32==3 || sellertypeu32==4 || sellertypeu32==5,Error::<T>::SellerTypeInvalid);
            // checking company name or name/surname
            let sellername=json_get_value(configuration.clone(),"name".as_bytes().to_vec());
            ensure!(sellername.len()>5,Error::<T>::SellerNameTooShort);
            // address we check for maximum lenght of 128 bytes
            let selleraddress=json_get_value(configuration.clone(),"address".as_bytes().to_vec());
            ensure!(selleraddress.len()<128,Error::<T>::SellerAddressTooLong);
            // zip code we check for maximum lenght of 12 bytes
            let sellerzip=json_get_value(configuration.clone(),"zip".as_bytes().to_vec());
            ensure!(sellerzip.len()<13,Error::<T>::SellerZipCodeTooLong);
            // checking the city minimum 3 bytes
            let sellerpobox=json_get_value(configuration.clone(),"pobox".as_bytes().to_vec());
            ensure!(sellerpobox.len()<64,Error::<T>::SellerPoBoxTooLong);
            // checking the city minimum 3 bytes
            let sellercity=json_get_value(configuration.clone(),"city".as_bytes().to_vec());
            ensure!(sellercity.len()>5,Error::<T>::SellerCityTooShort);
            ensure!(sellercity.len()<64,Error::<T>::SellerCityTooLong);
            // checking websites
            let websites=json_get_complexarray(configuration.clone(),"websites".as_bytes().to_vec());
            if websites.len()>0 {
                let mut x=0;
                loop {  
                    let w=json_get_recordvalue(websites.clone(),x);
                    if w.len()==0 {
                        break;
                    }
                    let weburl=json_get_value(w.clone(),"weburl".as_bytes().to_vec());
                    ensure!(aisland_validate_weburl(weburl)==true,Error::<T>::SellerWebsiteUrlIsWrong);
                    x=x+1;
                }
            }
            // checking social url
            let socialurls=json_get_complexarray(configuration.clone(),"socialurls".as_bytes().to_vec());
            if socialurls.len()>0 {
                let mut x=0;
                loop {  
                    let w=json_get_recordvalue(socialurls.clone(),x);
                    if w.len()==0 {
                        break;
                    }
                    let socialurl=json_get_value(w.clone(),"socialurl".as_bytes().to_vec());
                    ensure!(aisland_validate_weburl(socialurl)==true,Error::<T>::SellerSocialUrlIsWrong);
                    x=x+1;
                }
            }
            // checking certifications
            let certifications=json_get_complexarray(configuration.clone(),"certifications".as_bytes().to_vec());
            if certifications.len()>0 {
                let mut x=0;
                loop {  
                    let w=json_get_recordvalue(certifications.clone(),x);
                    if w.len()==0 {
                        break;
                    }
                    let certificationdescription=json_get_value(configuration.clone(),"description".as_bytes().to_vec());
                    let certificateverificationurl=json_get_value(configuration.clone(),"verificationurl".as_bytes().to_vec());
                    ensure!(certificationdescription.len()>3,Error::<T>::SellerCertificationDescriptionTooShort);
                    ensure!(certificationdescription.len()<=64,Error::<T>::SellerCertificationDescriptionTooLong);
                    ensure!(certificateverificationurl.len()>3,Error::<T>::SellerCertificateVerificationTooShort);
                    ensure!(certificateverificationurl.len()<=64,Error::<T>::SellerCertificateVerificationTooLong);
                    ensure!(aisland_validate_weburl(certificateverificationurl.clone())==true,Error::<T>::SellerCertificationUrlIsWrong);
                    x=x+1;
                }
            }
            // checking emailinfo
            let emailinfo=json_get_value(configuration.clone(),"emailinfo".as_bytes().to_vec());
            ensure!(emailinfo.len()>5,Error::<T>::SellerInfoEmailIsWrong);
            ensure!(aisland_validate_email(emailinfo.clone()),Error::<T>::SellerInfoEmailIsWrong);
            // checking email support
            let emailsupport=json_get_value(configuration.clone(),"emailsupport".as_bytes().to_vec());
            ensure!(emailsupport.len()>5,Error::<T>::SellerSupportEmailIsWrong);
            ensure!(aisland_validate_email(emailsupport.clone()),Error::<T>::SellerSupportEmailIsWrong);

            // checking phone numbers
            let phones=json_get_complexarray(configuration.clone(),"phones".as_bytes().to_vec());
            if phones.len()>0 {
                let mut x=0;
                loop {  
                    let w=json_get_recordvalue(phones.clone(),x);
                    if w.len()==0 {
                        break;
                    }
                    let phonedescription=json_get_value(configuration.clone(),"phonedescription".as_bytes().to_vec());
                    let phonenumber=json_get_value(configuration.clone(),"phonebumber".as_bytes().to_vec());
                    ensure!(phonedescription.len()>3,Error::<T>::SellerPhoneDescriptionTooShort);
                    ensure!(phonedescription.len()<=64,Error::<T>::SellerPhoneDescriptionTooLong);
                    ensure!(phonenumber.len()>3,Error::<T>::SellerPhoneNumberTooShort);
                    ensure!(phonenumber.len()<=23,Error::<T>::SellerPhoneNumberTooLong);
                    ensure!(aisland_validate_phonenumber(phonenumber)==true,Error::<T>::SellerPhoneNumberIsWrong);
                    x=x+1;
                }
            }
            // checking categories of products/services with the department
            let categories=json_get_complexarray(configuration.clone(),"categories".as_bytes().to_vec());
            ensure!(categories.len()>0,Error::<T>::SellerCategoriesMissing);
            let mut x=0;
            let mut nc=0;
            loop {  
                let c=json_get_recordvalue(categories.clone(),x);
                if c.len()==0 {
                    break;
                }
                let category=json_get_value(configuration.clone(),"category".as_bytes().to_vec());
                let department=json_get_value(configuration.clone(),"department".as_bytes().to_vec());
                let categoryu32=vecu8_to_u32(category);
                let departmentu32=vecu8_to_u32(department);
                ensure!(ProductCategories::contains_key(categoryu32,departmentu32)==true, Error::<T>::ProductCategoryNotFound);
                x=x+1;
                nc=nc+1;
            }
            // check that we have at least one valid product category
            ensure!(nc>0,Error::<T>::SellerCategoriesMissing);
            // checking included countries of shipment, if not set means worldwide less the excluded countries
            let countries=json_get_complexarray(configuration.clone(),"countries".as_bytes().to_vec());
            ensure!(countries.len()>0,Error::<T>::SellercountriesMissing);
            let mut x=0;
            loop {  
                let c=json_get_recordvalue(countries.clone(),x);
                if c.len()==0 {
                    break;
                }
                let country=json_get_value(configuration.clone(),"country".as_bytes().to_vec());
                let inout=json_get_value(configuration.clone(),"inout".as_bytes().to_vec());
                let inoutv=vecu8_to_u32(inout);
                ensure!(IsoCountries::contains_key(country)==true, Error::<T>::CountryCodeNotFound);
                ensure!(inoutv==0 || inoutv==1,Error::<T>::IncludedExcludedCountryValueIsMissing);
                x=x+1;
            }
            // check that we have at least one valid country
            ensure!(x>0,Error::<T>::SellerCategoriesMissing);
            // delivery area can be delimited by GPS coordinates where a first point is the center of a circle and second point is the border of the same circle
            // this is useful if a service/product can be delivered only around a certain place
            let shipmentarea=json_get_complexarray(configuration.clone(),"shipmentarea".as_bytes().to_vec());
            if shipmentarea.len()>0{
                let centerlatitude=json_get_value(shipmentarea.clone(),"centerlatitude".as_bytes().to_vec());
                let centerlongitude=json_get_value(shipmentarea.clone(),"centerlongitude".as_bytes().to_vec());
                let borderlatitude=json_get_value(shipmentarea.clone(),"borderlatitude".as_bytes().to_vec());
                let borderlongitude=json_get_value(shipmentarea.clone(),"borderlongitude".as_bytes().to_vec());
                ensure!(centerlatitude.len()>0,Error::<T>::ShipmentAreaCenterLatitudeIsMissing);
                ensure!(centerlongitude.len()>0,Error::<T>::ShipmentAreaCenterLongitudeIsMissing);
                ensure!(borderlatitude.len()>0,Error::<T>::ShipmentAreaBorderLatitudeIsMissing);
                ensure!(borderlongitude.len()>0,Error::<T>::ShipmentAreaBorderLongituteIsMissing);
            }
            // check for optional default language
            let defaultlanguage=json_get_value(configuration.clone(),"defaultlanguage".as_bytes().to_vec());
            if defaultlanguage.len()>0 {    
                ensure!(aisland_validate_languagecode(defaultlanguage),Error::<T>::SellerDefaultLanguageIsWrong);
            }
            // check for optional default unit of measurement
            let defaultunitmeasurement=json_get_value(configuration.clone(),"defaultunitmeasurement".as_bytes().to_vec());
            if defaultunitmeasurement.len()>0 {    
                ensure!(aisland_validate_unitmeasurement(defaultunitmeasurement),Error::<T>::SellerDefaultUnitMeasurementIsWrong);
            }
            // check for default return policy in days
            let defaultreturnpolicy=json_get_value(configuration.clone(),"defaultreturnpolicy".as_bytes().to_vec());
            if defaultreturnpolicy.len()>0 {    
                let drp=vecu8_to_u32(defaultreturnpolicy);
                ensure!(drp<3650,Error::<T>::DefaultReturnPolicyIsExcessive);
            }
            // check for optional seller account (when the signer acts as a proxy for gasless transactions)
            let selleraccount=json_get_value(configuration.clone(),"selleraccount".as_bytes().to_vec());
            if selleraccount.len()>0 {
                let selleraccountv=bs58::decode(selleraccount).into_vec().unwrap();
                let selleraccountid=T::AccountId::decode(&mut &selleraccountv[1..33]).unwrap_or_default();
                sender=selleraccountid;
            }
            //store seller on chain
            if Sellers::<T>::contains_key(&sender)==false {
                // Insert new seller
                Sellers::<T>::insert(sender.clone(),configuration.clone());
            } else {
                // check for proxy account before updating
                if originalsigner != sender {
                    let settings=Sellers::<T>::get(sender.clone()).unwrap();
                    let proxyaccount=json_get_value(settings.clone(),"proxyaccount".as_bytes().to_vec());
                    ensure!(proxyaccount.len()>0,Error::<T>::SignerIsNotAuthorized);
                    let proxyaccountv=bs58::decode(proxyaccount).into_vec().unwrap();
                    let proxyaccountid=T::AccountId::decode(&mut &proxyaccountv[1..33]).unwrap_or_default();
                    ensure!(proxyaccountid==originalsigner,Error::<T>::SignerIsNotAuthorized);
                }
                // Replace Seller Data 
                Sellers::<T>::take(sender.clone());
                Sellers::<T>::insert(sender.clone(),configuration.clone());
            }
            // Generate event
            Self::deposit_event(RawEvent::MarketPlaceSellerCreated(sender,configuration));
            // Return a successful DispatchResult
            Ok(())
        }
        /// Destroy a Seller
        #[weight = 1000]
        pub fn destroy_seller(origin) -> dispatch::DispatchResult {
            // check the request is signed
            let sender = ensure_signed(origin)?;
            // verify the seller exists
            ensure!(Sellers::<T>::contains_key(&sender)==true, Error::<T>::SellerDataNotFound);
            // Remove Seller
            Sellers::<T>::take(sender.clone());
            // Generate event
            //it can leave orphans, anyway it's a decision of the super user
            Self::deposit_event(RawEvent::MarketPlaceSellerDestroyed(sender));
            // Return a successful DispatchResult
            Ok(())
        }
        /// Create/update a Product
        /// Example:
        /// {"description":"xxxx","longdescription","xxxx","price":1000,"currencycode","USDC"}
        #[weight = 10000]
        pub fn create_update_product(origin, uid: u32, configuration: Vec<u8>) -> dispatch::DispatchResult {
            // check the request is signed
            let _sender = ensure_signed(origin)?;
            //check configuration length
            ensure!(configuration.len() > 12, Error::<T>::ConfigurationTooShort);
            ensure!(configuration.len() < 65536, Error::<T>::ConfigurationTooLong);
            // check json validity
            ensure!(json_check_validity(configuration.clone()),Error::<T>::InvalidJson);
            // check for mandatory short description
            let description=json_get_value(configuration.clone(),"description".as_bytes().to_vec());
            ensure!(description.len()>=10, Error::<T>::ProductDescriptionTooShort);
            ensure!(description.len()<=64, Error::<T>::ProductDescriptionTooLong);
            // check for mandatory long description
            let longdescription=json_get_value(configuration.clone(),"longdescription".as_bytes().to_vec());
            ensure!(longdescription.len()>=64, Error::<T>::ProductLongDescriptionTooShort);
            ensure!(longdescription.len()<=8192, Error::<T>::ProductLongDescriptionTooLong);
            // check for price >0
            let price=json_get_value(configuration.clone(),"price".as_bytes().to_vec());
            let pricevalue=vecu8_to_u128(price);
            ensure!(pricevalue>0,Error::<T>::ProductPriceCannotBeZero);
            // check for mandatory currency code
            let currencycode=json_get_value(configuration.clone(),"currency".as_bytes().to_vec());
            ensure!(Currencies::contains_key(&currencycode), Error::<T>::CurrencyCodeNotFound);
            // check for specifications
            let specifications=json_get_value(configuration.clone(),"specifications".as_bytes().to_vec());
            ensure!((specifications.len()==0 || specifications.len()>=8192),Error::<T>::SpecificationsIsdInvalid);
            // Media is an array of photos,videos and document being part of the product documentation
            // the structure can be [{"description":"xxxxx","filename":"xxxxxxx"},"ipfs":"xxxxxxx",("color":xx)},{..}]
            let media=json_get_complexarray(configuration.clone(),"media".as_bytes().to_vec());
            ensure!(media.len()>10,Error::<T>::MediaCannotBeEmpty);
            let mut x=0;
            loop {
                let jr=json_get_recordvalue(media.clone(),x);
                if jr.len()==0 {
                    break;
                }
                let description=json_get_value(jr.clone(),"description".as_bytes().to_vec());
                ensure!(description.len() > 0, Error::<T>::MediaDescriptionIswrong); 
                let filename=json_get_value(jr.clone(),"filename".as_bytes().to_vec());
                ensure!(filename.len() > 0, Error::<T>::MediaFileNameIsWrong); 
                let ipfs=json_get_value(jr.clone(),"ipfs".as_bytes().to_vec());
                ensure!(ipfs.len()>=32, Error::<T>::MediaIpfsAddressIsWrong);
                let color=json_get_value(jr.clone(),"color".as_bytes().to_vec());
                if color.len()>0 {
                    let colorvalue=vecu8_to_u32(color);
                    ensure!(ProductColors::contains_key(&colorvalue), Error::<T>::ColorNotFound);
                }
                x=x+1;
            }
            // check colors if enabled
            let colors=json_get_complexarray(configuration.clone(),"colors".as_bytes().to_vec());
            if colors.len()>0 {
                x=0;
                loop {
                    let c=json_get_arrayvalue(colors.clone(),x);
                    if c.len()==0 {
                        break;
                    }
                    let cv=vecu8_to_u32(c);
                    ensure!(ProductColors::contains_key(&cv), Error::<T>::ColorNotFound);
                    x=x+1;
                }
            }
            // check size if enabled
            let sizes=json_get_complexarray(configuration.clone(),"sizes".as_bytes().to_vec());
            if sizes.len()>0 {
                x=0;
                loop {
                    let sz=json_get_arrayvalue(sizes.clone(),x);
                    if sz.len()==0 {
                        break;
                    }
                    let szv=vecu8_to_u32(sz);
                    ensure!(ProductSizes::contains_key(&szv), Error::<T>::ColorNotFound);
                    x=x+1;
                }
            }
            // check dimension (optional)
            let dimension=json_get_complexarray(configuration.clone(),"dimension".as_bytes().to_vec());
            if dimension.len()>0 {
                x=0;
                loop {
                    let jr=json_get_recordvalue(dimension.clone(),x);
                    if jr.len()==0 {
                        break;
                    }
                    // check for length
                    let l=json_get_value(jr.clone(),"length".as_bytes().to_vec());
                    let v=vecu8_to_u32(l);
                    ensure!(v>0, Error::<T>::DimensionWrongLength);
                    // check for wide
                    let w=json_get_value(jr.clone(),"wide".as_bytes().to_vec());
                    let v=vecu8_to_u32(w);
                    ensure!(v>0, Error::<T>::DimensionWrongWide);
                    // check for height
                    let h=json_get_value(jr.clone(),"height".as_bytes().to_vec());
                    let v=vecu8_to_u32(h);
                    ensure!(v>0, Error::<T>::DimensionWrongHeight);
                    // check for Weight
                    let w=json_get_value(jr.clone(),"weight".as_bytes().to_vec());
                    let v=vecu8_to_u32(w);
                    ensure!(v>0, Error::<T>::DimensionWrongWeight);
                    x=x+1;
                }
            }
            // check UPC code (TODO - UPC check CRC validity)
            let u=json_get_value(configuration.clone(),"upc".as_bytes().to_vec());
            ensure!(u.len()>7, Error::<T>::UniversalProductCodeIsWrong);
            // check for shipping countries (optional)
            let shippingcountries=json_get_complexarray(configuration.clone(),"shippingcountries".as_bytes().to_vec());
            if shippingcountries.len()>0 {
                x=0;
                loop {
                    let countrycode=json_get_arrayvalue(shippingcountries.clone(),x);
                    if countrycode.len()==0 {
                        break;
                    }
                    ensure!(!IsoCountries::contains_key(&countrycode), Error::<T>::CountryCodeNotFound);
                    x=x+1;
                }
            }
            // check for shipping area
            // TODO (check for GPS coordinates validity)
            let shippingarea=json_get_complexarray(configuration.clone(),"shippingarea".as_bytes().to_vec());
            if shippingarea.len()>0 {
                x=0;
                loop {
                    let centerlatitude=json_get_value(shippingarea.clone(),"centerlatitude".as_bytes().to_vec());
                    let centerlongitude=json_get_value(shippingarea.clone(),"centerlongitude".as_bytes().to_vec());
                    let borderlatitude=json_get_value(shippingarea.clone(),"borderlatitude".as_bytes().to_vec());
                    let borderlongitude=json_get_value(shippingarea.clone(),"borderlongitude".as_bytes().to_vec());
                    ensure!(!centerlatitude.len()>0, Error::<T>::CenterLatitudeIsMissing);
                    ensure!(!centerlongitude.len()>0, Error::<T>::CenterLongitudeIsMissing);
                    ensure!(!borderlatitude.len()>0, Error::<T>::BorderLatitudeIsMissing);
                    ensure!(!borderlongitude.len()>0, Error::<T>::BorderLongitudeIsMissing);
                    x=x+1;
                }
            }
            // check for the shippers (optional field)
            let shippers=json_get_value(configuration.clone(),"shippers".as_bytes().to_vec());
            if shippers.len()>0 {
                x=0;
                loop {
                    let shipper=json_get_arrayvalue(shippers.clone(),x);
                    let v=vecu8_to_u32(shipper);
                    ensure!(Shippers::contains_key(&v), Error::<T>::ShipperNotFound);
                    x=x+1;
                }
            }
            // check for the API (optional field)
            let apiavailability=json_get_value(configuration.clone(),"apiavailability".as_bytes().to_vec());
            if apiavailability.len()>0 {
                ensure!(aisland_validate_weburl(apiavailability),Error::<T>::InvalidApiUrl);
            }
            // check for the language if any
            let language=json_get_value(configuration.clone(),"language".as_bytes().to_vec());
            if language.len()>0 {
                ensure!(aisland_validate_languagecode(language),Error::<T>::LanguageCodeIsWrong);
             }
            // TODO check the products was created from the same signer
            if Products::contains_key(&uid) {
                Products::take(&uid);
            }
            Products::insert(&uid,configuration.clone());
             // Generate event
             Self::deposit_event(RawEvent::MarketPlaceProductUpdated(uid,configuration));
            // Return a successful DispatchResult
            Ok(())
        }
    
        /// Create a new Iso country code and name
        #[weight = 1000]
        pub fn create_iso_country(origin, countrycode: Vec<u8>, countryname: Vec<u8>) -> dispatch::DispatchResult {
            // check the request is signed from the Super User
            let _sender = ensure_root(origin)?;
            // check country code length == 2
            ensure!(countrycode.len()==2, Error::<T>::WrongLengthCountryCode);
            // check country name length  >= 3
            ensure!(countryname.len()>=3, Error::<T>::CountryNameTooShort);
            // check the country is not alreay present on chain
            ensure!(IsoCountries::contains_key(&countrycode)==false, Error::<T>::CountryCodeAlreadyPresent);
            // store the Iso Country Code and Name
            IsoCountries::insert(countrycode.clone(),countryname.clone());
            // Generate event
            Self::deposit_event(RawEvent::MarketPlaceIsoCountryCreated(countrycode,countryname));
            // Return a successful DispatchResult
            Ok(())
        }
        /// Destroy an Iso country code and name
        #[weight = 1000]
        pub fn destroy_iso_country(origin, countrycode: Vec<u8>,) -> dispatch::DispatchResult {
            // check the request is signed from the Super User
            let _sender = ensure_root(origin)?;
            // verify the country code exists
            ensure!(IsoCountries::contains_key(&countrycode)==true, Error::<T>::CountryCodeNotFound);
            // Remove country code
            IsoCountries::take(countrycode.clone());
            // Generate event
            //it can leave orphans, anyway it's a decision of the super user
            Self::deposit_event(RawEvent::MarketPlaceIsoCountryDestroyed(countrycode));
            // Return a successful DispatchResult
            Ok(())
        }
         /// Create a new Iso dial code and name
         #[weight = 1000]
         pub fn create_dialcode_country(origin, countrycode: Vec<u8>, dialcode: Vec<u8>) -> dispatch::DispatchResult {
             // check the request is signed from the Super User
             let _sender = ensure_root(origin)?;
             // check country code length == 2
             ensure!(countrycode.len()==2, Error::<T>::WrongLengthCountryCode);
             // check country name length  >= 3
             ensure!(dialcode.len()>=2, Error::<T>::DialcodeTooShort);
             // check the dialcode is not alreay present on chain
             ensure!(IsoDialcode::contains_key(&countrycode)==false, Error::<T>::CountryCodeAlreadyPresent);
             // store the Iso Dial Code
             IsoDialcode::insert(countrycode.clone(),dialcode.clone());
             // Generate event
             Self::deposit_event(RawEvent::MarketPlaceIsoDialCodeCreated(countrycode,dialcode));
             // Return a successful DispatchResult
             Ok(())
         }
         /// Destroy an Iso country code and name
         #[weight = 1000]
         pub fn destroy_dialcode_country(origin, countrycode: Vec<u8>,) -> dispatch::DispatchResult {
             // check the request is signed from the Super User
             let _sender = ensure_root(origin)?;
             // verify the country code exists
             ensure!(IsoDialcode::contains_key(&countrycode)==true, Error::<T>::CountryCodeNotFound);
             // Remove country code
             IsoDialcode::take(countrycode.clone());
             // Generate event
             //it can leave orphans, anyway it's a decision of the super user
             Self::deposit_event(RawEvent::MarketPlaceIsoDialCodeDestroyed(countrycode));
             // Return a successful DispatchResult
             Ok(())
         }
         /// Create a new Currency code with name and other info in a json structure
         /// {"name":"Bitcoin","category":"c(rypto)/f(iat)","country":"countryisocode","blockchain":"Ethereum(...)","address":"xxxfor_crypto_currencyxxx"}
         /// for example: {"name":"Bitcoin","category":"c","country":"AE","blockchain":"Bitcoin","address":"not applicable"}
         /// {"name":"American Dollars","category":"f","country":"US","blockchain":"not applicable","address":"not applicable"}
        #[weight = 1000]
        pub fn create_currency(origin, currencycode: Vec<u8>, info: Vec<u8>) -> dispatch::DispatchResult {
            // check the request is signed from the Super User
            let _sender = ensure_root(origin)?;
            // check currency code length is between 3 and 5 bytes
            ensure!((currencycode.len()>=3 && currencycode.len()<=5), Error::<T>::WrongLengthCurrencyCode);
            // check the info field is not longer 1024 bytes
            ensure!((info.len()<=1024), Error::<T>::SizeInfoTooLong);
            // check for a valid json structure
            ensure!(json_check_validity(info.clone()),Error::<T>::InvalidJson);
            // check for name
            let name=json_get_value(info.clone(),"name".as_bytes().to_vec());
            ensure!(name.len()>=3, Error::<T>::CurrencyNameTooShort);
            ensure!(name.len()<=32, Error::<T>::CurrencyNameTooLong);
            // check for type of currency (fiat/crypto)
            let category=json_get_value(info.clone(),"category".as_bytes().to_vec());
            let mut c: Vec<u8>= Vec::new();
            c.push(b'c');
            let mut f: Vec<u8>= Vec::new();
            f.push(b'f');
            ensure!((category==c || category==f),Error::<T>::CurrencyCategoryIswrong);
            // check for the country code in case of Fiat currency
            if category==f {
                let countrycode=json_get_value(info.clone(),"country".as_bytes().to_vec());
                ensure!(IsoCountries::contains_key(&countrycode), Error::<T>::CountryCodeNotFound);
            }
            // check for the blockchain in case of Crypto currency
            if category==c {
                let blockchain=json_get_value(info.clone(),"blockchain".as_bytes().to_vec());
                ensure!(blockchain.len()>=3, Error::<T>::BlockchainNameTooShort);
                ensure!(blockchain.len()<=32, Error::<T>::BlockchainNameTooLong);
            }
            // check the currency is not alreay present on chain
            ensure!(!Currencies::contains_key(&currencycode), Error::<T>::CurrencyCodeAlreadyPresent);
            // store the Currency Code and info
            Currencies::insert(currencycode.clone(),info.clone());
            // Generate event
            Self::deposit_event(RawEvent::MarketPlaceCurrencyCodeCreated(currencycode,info));
            // Return a successful DispatchResult
            Ok(())
        }
        /// Destroy a currency
        #[weight = 1000]
        pub fn destroy_currency(origin, currencycode: Vec<u8>,) -> dispatch::DispatchResult {
            // check the request is signed from the Super User
            let _sender = ensure_root(origin)?;
            // verify the currency code exists
            ensure!(Currencies::contains_key(&currencycode), Error::<T>::CurrencyCodeNotFound);
            // Remove currency code
            Currencies::take(currencycode.clone());
            // Generate event
            //it can leave orphans, anyway it's a decision of the super user
            Self::deposit_event(RawEvent::MarketPlaceCurrencyDestroyed(currencycode));
            // Return a successful DispatchResult
            Ok(())
        }
        /// Create a new product Color
        #[weight = 1000]
        pub fn create_product_color(origin, uid: u32, description: Vec<u8>) -> dispatch::DispatchResult {
            // check the request is signed from root
            let _sender = ensure_root(origin)?;
            // check uid >0
            ensure!(uid > 0, Error::<T>::ColorUidCannotBeZero);
            //check description length
            ensure!(description.len() >= 2, Error::<T>::ColorDescriptionTooShort);
            ensure!(description.len() < 32, Error::<T>::ColorDescriptionTooLong);
            // check the color is not alreay present on chain
            ensure!(ProductColors::contains_key(uid)==false, Error::<T>::ColorAlreadyPresent);
            // store the color
            ProductColors::insert(uid,description.clone());
            // Generate event
            Self::deposit_event(RawEvent::MarketPlaceColorCreated(uid,description));
            // Return a successful DispatchResult
            Ok(())
        }
        /// Destroy a product color
        #[weight = 1000]
        pub fn destroy_product_color(origin, uid: u32) -> dispatch::DispatchResult {
            // check the request is signed from Super User
            let _sender = ensure_root(origin)?;
            // verify the color exists
            ensure!(ProductColors::contains_key(&uid)==true, Error::<T>::ColorNotFound);
            // Remove color
            ProductColors::take(uid);
            // Generate event
            //it can leave orphans, anyway it's a decision of the super user
            Self::deposit_event(RawEvent::MarketPlaceColorDestroyed(uid));
            // Return a successful DispatchResult
            Ok(())
        }
        /// Create a new product Size
        /// example json in info field: {"code":"XL","description":"Extra Large","area":"Europe"}
        #[weight = 1000]
        pub fn create_product_size(origin, uid: u32, info: Vec<u8>) -> dispatch::DispatchResult {
            // check the request is signed from root
            let _sender = ensure_root(origin)?;
            // check uid >0
            ensure!(uid > 0, Error::<T>::SizeUidCannotBeZero);
            //check info length
            ensure!(info.len() < 8192, Error::<T>::SizeInfoTooLong);
            // check valid json
            ensure!(json_check_validity(info.clone()),Error::<T>::InvalidJson);
            // checking sizes structure that must have some fields defined
            let mut x=0;
            loop {  
                let sz=json_get_recordvalue(info.clone(),x);
                if sz.len()==0 {
                    break;
                }
                let code=json_get_value(info.clone(),"code".as_bytes().to_vec());
                ensure!(code.len()>0,Error::<T>::SizeCodeIsMissing);
                let description=json_get_value(info.clone(),"description".as_bytes().to_vec());
                ensure!(description.len()>0,Error::<T>::SizeDescriptionIsMissing);
                let area=json_get_value(info.clone(),"area".as_bytes().to_vec());
                ensure!(area.len()>0,Error::<T>::SizeAreaIsMissing);
                x=x+1;
            }
            // check the size is not alreay present on chain
            ensure!(!ProductSizes::contains_key(uid), Error::<T>::SizeAlreadyPresent);
            // store the Size
            ProductSizes::insert(uid,info.clone());
            // Generate event
            Self::deposit_event(RawEvent::MarketPlaceSizeCreated(uid,info));
            // Return a successful DispatchResult
            Ok(())
        }
        /// Destroy a product size
        #[weight = 1000]
        pub fn destroy_product_size(origin, uid: u32) -> dispatch::DispatchResult {
            // check the request is signed from Super User
            let _sender = ensure_root(origin)?;
            // verify the size exists
            ensure!(ProductSizes::contains_key(&uid)==true, Error::<T>::SizeNotFound);
            // Remove size
            ProductSizes::take(uid);
            // Generate event
            //it can leave orphans, anyway it's a decision of the super user
            Self::deposit_event(RawEvent::MarketPlaceSizeDestroyed(uid));
            // Return a successful DispatchResult
            Ok(())
        }
        /// Create a new Manufacturer
        /// Example field info: {"name":"Samsung","website":"https://www.samsung.com"}
        #[weight = 1000]
        pub fn create_manufacturer(origin, uid: u32, info: Vec<u8>,) -> dispatch::DispatchResult {
            // check the request is signed from root
            let _sender = ensure_root(origin)?;
            // check uid >0
            ensure!(uid > 0, Error::<T>::ManufacturerUidCannotBeZero);
            // check valid json
            ensure!(json_check_validity(info.clone()),Error::<T>::InvalidJson);
            // check for name field
            let name=json_get_value(info.clone(),"name".as_bytes().to_vec());
            ensure!(name.len()>=4,Error::<T>::ManufacturerNameIsTooShort);
            ensure!(name.len()<=64,Error::<T>::ManufacturerNameIsTooLong);
            // check for website field
            let website=json_get_value(info.clone(),"website".as_bytes().to_vec());
            ensure!(website.len()>=4,Error::<T>::ManufacturerWebsiteIsTooShort);
            ensure!(website.len()<=64,Error::<T>::ManufacturerWebsiteIsTooLong);
            // check the manufacturer is not alreay present on chain
            ensure!(!Manufacturers::contains_key(uid), Error::<T>::ManufacturerAlreadyPresent);
            // store the manufacturer
            Manufacturers::insert(uid,info.clone());
            // Generate event
            Self::deposit_event(RawEvent::MarketPlaceManufacturerCreated(uid,info));
            // Return a successful DispatchResult
            Ok(())
        }
        /// Destroy a manufacturer 
        #[weight = 1000]
        pub fn destroy_manufacturer(origin, uid: u32) -> dispatch::DispatchResult {
            // check the request is signed from Super User
            let _sender = ensure_root(origin)?;
            // verify the manufacturer exists
            ensure!(Manufacturers::contains_key(&uid), Error::<T>::ManufacturerNotFound);
            // Remove manufacturer
            Manufacturers::take(uid);
            // Generate event
            //it can leave orphans, anyway it's a decision of the super user
            Self::deposit_event(RawEvent::MarketPlaceManufacturerDestroyed(uid));
            // Return a successful DispatchResult
            Ok(())
        }
        /// Create a new Shipper
        /// exmaple info field: {"name":"DHL","website":"www.dhl.com"}
        #[weight = 1000]
        pub fn create_shipper(origin, uid: u32, info: Vec<u8>,) -> dispatch::DispatchResult {
            // check the request is signed from root
            let _sender = ensure_root(origin)?;
            // check uid >0
            ensure!(uid > 0, Error::<T>::ShipperUidCannotBeZero);
            // check valid json
            ensure!(json_check_validity(info.clone()),Error::<T>::InvalidJson);
            ensure!(info.len()<=16384,Error::<T>::InvalidJson);
            // check for name field
            let name=json_get_value(info.clone(),"name".as_bytes().to_vec());
            ensure!(name.len()>=3,Error::<T>::ShipperNameIsTooShort);
            ensure!(name.len()<=64,Error::<T>::ShipperNameIsTooLong);
            // check for info field
            ensure!(info.len()<=32768,Error::<T>::JsonIsTooLong);
            // check for website field
            let website=json_get_value(info.clone(),"website".as_bytes().to_vec());
            ensure!(website.len()>=4,Error::<T>::ShipperWebsiteIsTooShort);
            ensure!(website.len()<=64,Error::<T>::ShipperWebsiteIsTooLong);
            // check the Shipper is not alreay present on chain
            ensure!(!Shippers::contains_key(uid), Error::<T>::ShipperAlreadyPresent);
            // check for origincountries field (optional)
            let origincountries=json_get_complexarray(info.clone(),"origincountries".as_bytes().to_vec());
            if origincountries.len()>0 {
                let mut x=0;
                loop {  
                    let oc=json_get_arrayvalue(origincountries.clone(),x);
                    if oc.len()==0 {
                        break;
                    }
                    ensure!(IsoCountries::contains_key(oc),Error::<T>::OriginCountryNotPresent);
                    x=x+1;
                }
            }
            // check for destinationcountries field (optional)
            let destinationcountries=json_get_complexarray(info.clone(),"destinationcountries".as_bytes().to_vec());
            if destinationcountries.len()>0 {
                let mut x=0;
                loop {  
                    let dc=json_get_arrayvalue(destinationcountries.clone(),x);
                    if dc.len()==0 {
                        break;
                    }
                    ensure!(IsoCountries::contains_key(dc),Error::<T>::DestinationCountryNotPresent);
                    x=x+1;
                }
            }
            // store the shippers
            Shippers::insert(uid,info.clone());
            // Generate event
            Self::deposit_event(RawEvent::MarketPlaceShipperCreated(uid,info));
            // Return a successful DispatchResult
            Ok(())
        }
        /// Destroy a Shipper 
        #[weight = 1000]
        pub fn destroy_shipper(origin, uid: u32) -> dispatch::DispatchResult {
            // check the request is signed from Super User
            let _sender = ensure_root(origin)?;
            // verify the shipper exists
            ensure!(Shippers::contains_key(&uid), Error::<T>::ShipperNotFound);
            // Remove shipper
            Shippers::take(uid);
            // Generate event
            //it can leave orphans, anyway it's a decision of the super user
            Self::deposit_event(RawEvent::MarketPlaceShipperDestroyed(uid));
            // Return a successful DispatchResult
            Ok(())
        }
          /// Create new Shipping Rates
          /// example field info: {"shipperid":1,"origincountry":"AE","currency":"AED","rates":[{"destination":"LR","fromkg":0,"to":1,"rate":10},{"destination":"LR","fromkg":1,"tokg":2,"rate":15}]}
          #[weight = 1000]
          pub fn create_shipping_rates(origin, uid: u32, info: Vec<u8>,) -> dispatch::DispatchResult {
            // check the request is signed from root
            let _sender = ensure_root(origin)?;
            // check uid >0
            ensure!(uid > 0, Error::<T>::ShippingRateUidCannotBeZero);
            // check valid json
            ensure!(json_check_validity(info.clone()),Error::<T>::InvalidJson);
            // check for info field
            ensure!(info.len()<=32768,Error::<T>::JsonIsTooLong);
            // check for shipperid field
            let shipperid=json_get_value(info.clone(),"shipperid".as_bytes().to_vec());
            let shipperidv=vecu8_to_u32(shipperid);
            ensure!(shipperidv>0,Error::<T>::ShipperIdIsMissing);   
            ensure!(Shippers::contains_key(shipperidv),Error::<T>::ShipperNotFound);
            // check for origincountry field
            let origincountry=json_get_value(info.clone(),"origincountry".as_bytes().to_vec());
            ensure!(IsoCountries::contains_key(origincountry.clone()),Error::<T>::OriginCountryNotPresent);
            // check for currency field
            let currency=json_get_value(info.clone(),"currency".as_bytes().to_vec());
            ensure!(Currencies::contains_key(currency),Error::<T>::CurrencyCodeNotFound);
            // check for rates 
            let rates=json_get_complexarray(info.clone(),"rates".as_bytes().to_vec());
            if rates.len()>0 {
                let mut x=0;
                loop {  
                    let r=json_get_recordvalue(rates.clone(),x);
                    if r.len()==0 {
                        break;
                    }
                    // check for destination
                    let dc=json_get_value(r.clone(),"destination".as_bytes().to_vec());
                    ensure!(IsoCountries::contains_key(dc),Error::<T>::OriginCountryNotPresent);
                    // check for fromkg
                    let fromkg=json_get_value(r.clone(),"fromkg".as_bytes().to_vec());
                    ensure!(fromkg.len()>0,Error::<T>::FromKgIsMissing);
                    // check for tokg
                    let tokg=json_get_value(r.clone(),"tokg".as_bytes().to_vec());
                    ensure!(tokg.len()>0,Error::<T>::ToKgIsMissing);  
                    // check for rates  (in the currency set)
                    let rate=json_get_value(r.clone(),"rate".as_bytes().to_vec());
                    let ratev=vecu8_to_u32(rate);
                    ensure!(ratev>0,Error::<T>::ShippingRateCannotbeZero);  
                    x=x+1;
                }
            }
            // store the shipping rates
            ShippingRates::insert(uid,info.clone());
            // Generate event
            Self::deposit_event(RawEvent::MarketShippingRateCreated(uid,info));
            // Return a successful DispatchResult
            Ok(())
          }
          /// Destroy Shipping Rates 
          #[weight = 1000]
          pub fn destroy_shipping_rates(origin, uid: u32) -> dispatch::DispatchResult {
              // check the request is signed from Super User
              let _sender = ensure_root(origin)?;
              // verify the shipping rate exists
              ensure!(ShippingRates::contains_key(&uid), Error::<T>::ShippingRatesNotFound);
              // Remove shipper
              Shippers::take(uid);
              // Generate event
              //it can leave orphans, anyway it's a decision of the super user
              Self::deposit_event(RawEvent::MarketPlaceShipperDestroyed(uid));
              // Return a successful DispatchResult
              Ok(())
          }
        /// Create a new Brand
        /// Example of info field: {"name":"Galaxy","manufacturer":7}
        #[weight = 1000]
        pub fn create_brand(origin, uid: u32, info: Vec<u8>,) -> dispatch::DispatchResult {
            // check the request is signed from root
            let _sender = ensure_root(origin)?;
            // check uid >0
            ensure!(uid > 0, Error::<T>::BrandUidCannotBeZero);
            ensure!(info.len()<=1024,Error::<T>::JsonIsTooLong);
            // check valid json
            ensure!(json_check_validity(info.clone()),Error::<T>::InvalidJson);
            // check for name field
            let name=json_get_value(info.clone(),"name".as_bytes().to_vec());
            ensure!(name.len()>=4,Error::<T>::BrandNameIsTooShort);
            ensure!(name.len()<=64,Error::<T>::BrandNameIsTooLong);
            // check for website field
            let manufacturer=json_get_value(info.clone(),"manufacturer".as_bytes().to_vec());
            let mv=vecu8_to_u32(manufacturer);
            ensure!(mv>0,Error::<T>::ManufacturerNotFound);
            // check the Manufacturer is  present on chain
            ensure!(Manufacturers::contains_key(mv), Error::<T>::ManufacturerNotFound);
            // check the brand is not present on chain
            ensure!(!Brands::contains_key(uid), Error::<T>::BrandAlreadyPresent);
            // store the brand
            Brands::insert(uid,info.clone());
            // Generate event
            Self::deposit_event(RawEvent::MarketPlaceBrandCreated(uid,info));
            // Return a successful DispatchResult
            Ok(())
        }
        /// Destroy a Brand
        #[weight = 1000]
        pub fn destroy_brand(origin, uid: u32) -> dispatch::DispatchResult {
            // check the request is signed from Super User
            let _sender = ensure_root(origin)?;
            // verify the brand exists
            ensure!(Brands::contains_key(&uid), Error::<T>::BrandNotFound);
            // Remove brand
            Brands::take(uid);
            // Generate event
            //it can leave orphans, anyway it's a decision of the super user
            Self::deposit_event(RawEvent::MarketPlaceBrandDestroyed(uid));
            // Return a successful DispatchResult
            Ok(())
        }
         /// Create a new product model
         /// Example field info: {"name":"A1","brand":1}
         #[weight = 1000]
         pub fn create_product_model(origin, uid: u32, info: Vec<u8>,) -> dispatch::DispatchResult {
             // check the request is signed from root
             let _sender = ensure_root(origin)?;
             // check uid >0
             ensure!(uid > 0, Error::<T>::ModelUidCannotBeZero);
             // check valid json
             ensure!(json_check_validity(info.clone()),Error::<T>::InvalidJson);
             // check for name field
             let name=json_get_value(info.clone(),"name".as_bytes().to_vec());
             ensure!(name.len()>=2,Error::<T>::ModelNameIsTooShort);
             ensure!(name.len()<=32,Error::<T>::ModelNameIsTooLong);
             // check for brand field
             let brand=json_get_value(info.clone(),"brand".as_bytes().to_vec());
             let bv=vecu8_to_u32(brand);
             ensure!(bv>0,Error::<T>::BrandNotFound);
             // check the brand is  present on chain
             ensure!(Brands::contains_key(bv), Error::<T>::BrandNotFound);
             // check the model is not present on chain
             ensure!(!ProductModels::contains_key(uid), Error::<T>::ModelAlreadyPresent);
             // store the model
             ProductModels::insert(uid,info.clone());
             // Generate event
             Self::deposit_event(RawEvent::MarketPlaceProductModelCreated(uid,info));
             // Return a successful DispatchResult
             Ok(())
         }
         /// Destroy a product model
         #[weight = 1000]
         pub fn destroy_product_model(origin, uid: u32) -> dispatch::DispatchResult {
             // check the request is signed from Super User
             let _sender = ensure_root(origin)?;
             // verify the model exists
             ensure!(ProductModels::contains_key(&uid), Error::<T>::BrandNotFound);
             // Remove model
             ProductModels::take(uid);
             // Generate event
             //it can leave orphans, anyway it's a decision of the super user
             Self::deposit_event(RawEvent::MarketPlaceProductModelDestroyed(uid));
             // Return a successful DispatchResult
             Ok(())
         }
         /// Create a new Login Data
        #[weight = 1000]
        pub fn create_login_data(origin, emailhash: Vec<u8>, encryptedpwdhash: Vec<u8>, accountid: T::AccountId,encryptedseed: Vec<u8>) -> dispatch::DispatchResult {
            // check the request is signed 
            let _sender = ensure_signed(origin)?;
            // check Email hash length
            ensure!(emailhash.len()>8, Error::<T>::WrongLengthEmailHash);
            // check Encrypted Password length
            ensure!(encryptedpwdhash.len()>8, Error::<T>::WrongLengthEncryptedPassword);
            // check the email ahsh is not alreay present on chain
            ensure!(LoginData::contains_key(&emailhash)==false, Error::<T>::EmailHashAlreadyPresent);
            // store the Login Data
            LoginData::insert(emailhash.clone(),encryptedpwdhash.clone());
            // store the Account id
            EmailAccount::<T>::insert(emailhash.clone(),accountid.clone());
            // store the encrypted seed (double encryption layer)
            EmailEncryptedSeed::insert(emailhash.clone(),encryptedseed.clone());
            // Generate event
            Self::deposit_event(RawEvent::MarketPlaceLoginDataCreated(emailhash,encryptedpwdhash,accountid));
            // Return a successful DispatchResult
            Ok(())
        }
        /// Create a new Login Data
        #[weight = 1000]
        pub fn change_pwd_login_data(origin, emailhash: Vec<u8>, encryptedpwdhash: Vec<u8>) -> dispatch::DispatchResult {
            // check the request is signed from the same user who created it
            let sender = ensure_signed(origin)?;
            // check Email hash length
            ensure!(emailhash.len()>8, Error::<T>::WrongLengthEmailHash);
            // check Encrypted Password length
            ensure!(encryptedpwdhash.len()>8, Error::<T>::WrongLengthEncryptedPassword);
            // check the email hash is alreay present on chain
            ensure!(LoginData::contains_key(&emailhash), Error::<T>::EmailHashNotFound);
            // check the email account is present on chain
            ensure!(EmailAccount::<T>::contains_key(&emailhash),Error::<T>::EmailHashNotFound);
            //let accountidemail=EmailAccount::<T>::get(&emailhash).unw;
            let accountidemail=EmailAccount::<T>::get(emailhash.clone());
            // check that the signer is the creator of the original state
            ensure!(sender==accountidemail,Error::<T>::SignerIsNotAuthorized);
            // store the Login Data
            LoginData::take(&emailhash);
            LoginData::insert(emailhash.clone(),encryptedpwdhash.clone());
            // Generate event
            Self::deposit_event(RawEvent::MarketPlaceLoginPwdChanged(emailhash,encryptedpwdhash));
            // Return a successful DispatchResult
            Ok(())
        }

        /// Destroy a login data
        #[weight = 1000]
        pub fn destroy_login_data(origin, emailhash: Vec<u8>,) -> dispatch::DispatchResult {
            // check the request is signed from the same signer of the original writing
            let sender = ensure_signed(origin)?;
            // verify the login data exists
            ensure!(LoginData::contains_key(&emailhash)==true, Error::<T>::EmailHashNotFound);
            ensure!(EmailAccount::<T>::contains_key(&emailhash)==true, Error::<T>::EmailHashNotFound);
            let accountid=EmailAccount::<T>::get(emailhash.clone());
            ensure!(accountid==sender,Error::<T>::SignerIsNotAuthorized);
            // Remove email hash and accountid and encrypted seed
            LoginData::take(emailhash.clone());
            EmailAccount::<T>::take(emailhash.clone());
            EmailEncryptedSeed::take(emailhash.clone());
            // Generate event
            //it can leave orphans, anyway it's a decision of the super user
            Self::deposit_event(RawEvent::MarketPlaceLoginDataDestroyed(emailhash));
            // Return a successful DispatchResult
            Ok(())
        }
    }
}


//*************************************************************************************
//*** functions blocks
//*************************************************************************************
// function to validate a json string for no/std. It does not allocate of memory
fn json_check_validity(j:Vec<u8>) -> bool{	
    // minimum lenght of 2
    if j.len()<2 {
        return false;
    }
    // checks star/end with {}
    if *j.get(0).unwrap()==b'{' && *j.get(j.len()-1).unwrap()!=b'}' {
        return false;
    }
    // checks start/end with []
    if *j.get(0).unwrap()==b'[' && *j.get(j.len()-1).unwrap()!=b']' {
        return false;
    }
    // check that the start is { or [
    if *j.get(0).unwrap()!=b'{' && *j.get(0).unwrap()!=b'[' {
            return false;
    }
    //checks that end is } or ]
    if *j.get(j.len()-1).unwrap()!=b'}' && *j.get(j.len()-1).unwrap()!=b']' {
        return false;
    }
    //checks " opening/closing and : as separator between name and values
    let mut s:bool=true;
    let mut d:bool=true;
    let mut pg:bool=true;
    let mut ps:bool=true;
    let mut bp = b' ';
    for b in j {
        if b==b'[' && s {
            ps=false;
        }
        if b==b']' && s && ps==false {
            ps=true;
        }
        else if b==b']' && s && ps==true {
            ps=false;
        }
        if b==b'{' && s {
            pg=false;
        }
        if b==b'}' && s && pg==false {
            pg=true;
        }
        else if b==b'}' && s && pg==true {
            pg=false;
        }
        if b == b'"' && s && bp != b'\\' {
            s=false;
            bp=b;
            d=false;
            continue;
        }
        if b == b':' && s {
            d=true;
            bp=b;
            continue;
        }
        if b == b'"' && !s && bp != b'\\' {
            s=true;
            bp=b;
            d=true;
            continue;
        }
        bp=b;
    }
    //fields are not closed properly
    if !s {
        return false;
    }
    //fields are not closed properly
    if !d {
        return false;
    }
    //fields are not closed properly
    if !ps {
        return false;
    }
    // every ok returns true
    return true;
}
// function to get record {} from multirecord json structure [{..},{.. }], it returns an empty Vec when the records is not present
fn json_get_recordvalue(ar:Vec<u8>,p:i32) -> Vec<u8> {
    let mut result=Vec::new();
    let mut op=true;
    let mut cn=0;
    let mut lb=b' ';
    for b in ar {
        if b==b',' && op==true {
            cn=cn+1;
            continue;
        }
        if b==b'[' && op==true && lb!=b'\\' {
            continue;
        }
        if b==b']' && op==true && lb!=b'\\' {
            continue;
        }
        if b==b'{' && op==true && lb!=b'\\' { 
            op=false;
        }
        if b==b'}' && op==false && lb!=b'\\' {
            op=true;
        }
        // field found
        if cn==p {
            result.push(b);
        }
        lb=b.clone();
    }
    return result;
}
// function to get a field value from array field [1,2,3,4,100], it returns an empty Vec when the records is not present
fn json_get_arrayvalue(ar:Vec<u8>,p:i32) -> Vec<u8> {
    let mut result=Vec::new();
    let mut op=true;
    let mut cn=0;
    let mut lb=b' ';
    for b in ar {
        if b==b',' && op==true {
            cn=cn+1;
            continue;
        }
        if b==b'[' && op==true && lb!=b'\\' {
            continue;
        }
        if b==b']' && op==true && lb!=b'\\' {
            continue;
        }
        if b==b'"' && op==true && lb!=b'\\' {
            continue;
        }
        if b==b'"' && op==true && lb!=b'\\' { 
            op=false;
        }
        if b==b'"' && op==false && lb!=b'\\' {
            op=true;
        }
        // field found
        if cn==p {
            result.push(b);
        }
        lb=b.clone();
    }
    return result;
}

// function to get value of a field for Substrate runtime (no std library and no variable allocation)
fn json_get_value(j:Vec<u8>,key:Vec<u8>) -> Vec<u8> {
    let mut result=Vec::new();
    let mut k=Vec::new();
    let keyl = key.len();
    let jl = j.len();
    k.push(b'"');
    for xk in 0..keyl{
        k.push(*key.get(xk).unwrap());
    }
    k.push(b'"');
    k.push(b':');
    let kl = k.len();
    for x in  0..jl {
        let mut m=0;
        let mut xx=0;
        if x+kl>jl {
            break;
        }
        for i in x..x+kl {
            if *j.get(i).unwrap()== *k.get(xx).unwrap() {
                m=m+1;
            }
            xx=xx+1;
        }
        if m==kl{
            let mut lb=b' ';
            let mut op=true;
            let mut os=true;
            for i in x+kl..jl-1 {
                if *j.get(i).unwrap()==b'[' && op==true && os==true{
                    os=false;
                }
                if *j.get(i).unwrap()==b'}' && op==true && os==false{
                    os=true;
                }
                if *j.get(i).unwrap()==b':' && op==true{
                    continue;
                }
                if *j.get(i).unwrap()==b'"' && op==true && lb!=b'\\' {
                    op=false;
                    continue
                }
                if *j.get(i).unwrap()==b'"' && op==false && lb!=b'\\' {
                    break;
                }
                if *j.get(i).unwrap()==b'}' && op==true{
                    break;
                }
                if *j.get(i).unwrap()==b']' && op==true{
                    break;
                }
                if *j.get(i).unwrap()==b',' && op==true && os==true{
                    break;
                }
                result.push(j.get(i).unwrap().clone());
                lb=j.get(i).unwrap().clone();
            }   
            break;
        }
    }
    return result;
}
// function to get value of a field with a complex array like [{....},{.....}] for Substrate runtime (no std library and no variable allocation)
fn json_get_complexarray(j:Vec<u8>,key:Vec<u8>) -> Vec<u8> {
    let mut result=Vec::new();
    let mut k=Vec::new();
    let keyl = key.len();
    let jl = j.len();
    k.push(b'"');
    for xk in 0..keyl{
        k.push(*key.get(xk).unwrap());
    }
    k.push(b'"');
    k.push(b':');
    let kl = k.len();
    for x in  0..jl {
        let mut m=0;
        let mut xx=0;
        if x+kl>jl {
            break;
        }
        for i in x..x+kl {
            if *j.get(i).unwrap()== *k.get(xx).unwrap() {
                m=m+1;
            }
            xx=xx+1;
        }
        if m==kl{
            let mut os=true;
            for i in x+kl..jl-1 {
                if *j.get(i).unwrap()==b'[' && os==true{
                    os=false;
                }
                result.push(j.get(i).unwrap().clone());
                if *j.get(i).unwrap()==b']' && os==false {
                    break;
                }
            }   
            break;
        }
    }
    return result;
}
// function to validate and email address, return true/false
fn aisland_validate_email(email:Vec<u8>) -> bool {
    let mut flagat=false;
    let mut valid=false;
    let mut phase=1;
    let mut dotphase2=false;
    for c in email {
        if c==64 {
            flagat=true;
            phase=2;
            continue;
        }
        // check for allowed char in the first part of the email address before @
        if phase==1 {
            if  (c>=65 && c<=90) ||
                (c>=97 && c<=122) ||
                c==45 || c==46 || c==95 {
                    valid=true;
                }else
                {
                    valid=false;
                    break;
                }
        }   
        // check for allowed char in the second part of the email address before @
        if phase==2 {
            if  (c>=65 && c<=90) ||
                (c>=97 && c<=122) ||
                c==45 || c==46 {
                    valid=true;
                }else
                {
                    valid=false;
                    break;
                }
                if c==46 {
                    dotphase2=true;
                }
        }

    }
    // return validity true/false
    if flagat==true && dotphase2==true{
        return valid;
    }else {
        return flagat;
    }   
}
// function to validate an web url return true/false
fn aisland_validate_weburl(weburl:Vec<u8>) -> bool {
    let mut valid=false;
    let mut x=0;
    let mut httpsflag=false;
    let mut httpflag=false;
    let mut startpoint=0;
    let mut https: Vec<u8>= Vec::new();
    https.push(b'h');
    https.push(b't');
    https.push(b't');
    https.push(b'p');
    https.push(b's');
    https.push(b':');
    https.push(b'/');
    https.push(b'/');
    let mut http: Vec<u8>= Vec::new();
    http.push(b'h');
    http.push(b't');
    http.push(b't');
    http.push(b'p');
    http.push(b':');
    http.push(b'/');
    http.push(b'/');
    let mut httpscomp: Vec<u8> =Vec::new();
    httpscomp.push(weburl[0]);
    httpscomp.push(weburl[1]);
    httpscomp.push(weburl[2]);
    httpscomp.push(weburl[3]);
    httpscomp.push(weburl[4]);
    httpscomp.push(weburl[5]);
    httpscomp.push(weburl[6]);
    httpscomp.push(weburl[7]);
    let mut httpcomp: Vec<u8> =Vec::new();
    httpcomp.push(weburl[0]);
    httpcomp.push(weburl[1]);
    httpcomp.push(weburl[2]);
    httpcomp.push(weburl[3]);
    httpcomp.push(weburl[4]);
    httpcomp.push(weburl[5]);
    httpcomp.push(weburl[6]);
    if https==httpscomp {
        httpsflag=true;
    }
    if http==httpcomp {
        httpflag=true;
    }
    if httpflag==false && httpsflag==false {
        return false;
    }
    if httpsflag==true{
        startpoint=8;
    }
    if httpflag==true{
        startpoint=7;
    }
    for c in weburl {    
        if x<startpoint {
            x=x+1;
            continue;
        }
        // check for allowed chars    
        if  (c>=32 && c<=95) ||
            (c>=97 && c<=126) {
            valid=true;
        }else{
            valid=false;
            break;
        }
    }
    return valid;
}
// function to validate a phone number
fn aisland_validate_phonenumber(phonenumber:Vec<u8>) -> bool {
    // check maximum lenght
    if phonenumber.len()>23{
        return false;
    }
    // check admitted bytes
    let mut x=0;
    for v in phonenumber.clone() {
        if (v>=48 && v<=57) || (v==43 && x==0){
            x=x+1;
        }else {
            return false;
        }
    }
    // load international prefixes table
    let mut p: Vec<Vec<u8>> = Vec::new();
    p.push("972".into());
    p.push("93".into());
    p.push("355".into());
    p.push("213".into());
    p.push("376".into());
    p.push("244".into());
    p.push("54".into());
    p.push("374".into());
    p.push("297".into());
    p.push("61".into());
    p.push("43".into());
    p.push("994".into());
    p.push("973".into());
    p.push("880".into());
    p.push("375".into());
    p.push("32".into());
    p.push("501".into());
    p.push("229".into());
    p.push("975".into());
    p.push("387".into());
    p.push("267".into());
    p.push("55".into());
    p.push("246".into());
    p.push("359".into());
    p.push("226".into());
    p.push("257".into());
    p.push("855".into());
    p.push("237".into());
    p.push("1".into());
    p.push("238".into());
    p.push("345".into());
    p.push("236".into());
    p.push("235".into());
    p.push("56".into());
    p.push("86".into());
    p.push("61".into());
    p.push("57".into());
    p.push("269".into());
    p.push("242".into());
    p.push("682".into());
    p.push("506".into());
    p.push("385".into());
    p.push("53".into());
    p.push("537".into());
    p.push("420".into());
    p.push("45".into());
    p.push("253".into());
    p.push("593".into());
    p.push("20".into());
    p.push("503".into());
    p.push("240".into());
    p.push("291".into());
    p.push("372".into());
    p.push("251".into());
    p.push("298".into());
    p.push("679".into());
    p.push("358".into());
    p.push("33".into());
    p.push("594".into());
    p.push("689".into());
    p.push("241".into());
    p.push("220".into());
    p.push("995".into());
    p.push("49".into());
    p.push("233".into());
    p.push("350".into());
    p.push("30".into());
    p.push("299".into());
    p.push("590".into());
    p.push("502".into());
    p.push("224".into());
    p.push("245".into());
    p.push("595".into());
    p.push("509".into());
    p.push("504".into());
    p.push("36".into());
    p.push("354".into());
    p.push("91".into());
    p.push("62".into());
    p.push("964".into());
    p.push("353".into());
    p.push("972".into());
    p.push("39".into());
    p.push("81".into());
    p.push("962".into());
    p.push("254".into());
    p.push("686".into());
    p.push("965".into());
    p.push("996".into());
    p.push("371".into());
    p.push("961".into());
    p.push("266".into());
    p.push("231".into());
    p.push("423".into());
    p.push("370".into());
    p.push("352".into());
    p.push("261".into());
    p.push("265".into());
    p.push("60".into());
    p.push("960".into());
    p.push("223".into());
    p.push("356".into());
    p.push("692".into());
    p.push("596".into());
    p.push("222".into());
    p.push("230".into());
    p.push("262".into());
    p.push("52".into());
    p.push("377".into());
    p.push("976".into());
    p.push("382".into());
    p.push("1664".into());
    p.push("212".into());
    p.push("95".into());
    p.push("264".into());
    p.push("674".into());
    p.push("977".into());
    p.push("31".into());
    p.push("599".into());
    p.push("687".into());
    p.push("64".into());
    p.push("505".into());
    p.push("227".into());
    p.push("234".into());
    p.push("683".into());
    p.push("672".into());
    p.push("47".into());
    p.push("968".into());
    p.push("92".into());
    p.push("680".into());
    p.push("507".into());
    p.push("675".into());
    p.push("595".into());
    p.push("51".into());
    p.push("63".into());
    p.push("48".into());
    p.push("351".into());
    p.push("974".into());
    p.push("40".into());
    p.push("250".into());
    p.push("685".into());
    p.push("378".into());
    p.push("966".into());
    p.push("221".into());
    p.push("381".into());
    p.push("248".into());
    p.push("232".into());
    p.push("65".into());
    p.push("421".into());
    p.push("386".into());
    p.push("677".into());
    p.push("27".into());
    p.push("500".into());
    p.push("34".into());
    p.push("94".into());
    p.push("249".into());
    p.push("597".into());
    p.push("268".into());
    p.push("46".into());
    p.push("41".into());
    p.push("992".into());
    p.push("66".into());
    p.push("228".into());
    p.push("690".into());
    p.push("676".into());
    p.push("216".into());
    p.push("90".into());
    p.push("993".into());
    p.push("688".into());
    p.push("256".into());
    p.push("380".into());
    p.push("971".into());
    p.push("44".into());
    p.push("1".into());
    p.push("598".into());
    p.push("998".into());
    p.push("678".into());
    p.push("681".into());
    p.push("967".into());
    p.push("260".into());
    p.push("263".into());
    p.push("591".into());
    p.push("673".into());
    p.push("61".into());
    p.push("243".into());
    p.push("225".into());
    p.push("500".into());
    p.push("44".into());
    p.push("379".into());
    p.push("852".into());
    p.push("98".into());
    p.push("44".into());
    p.push("44".into());
    p.push("850".into());
    p.push("82".into());
    p.push("856".into());
    p.push("218".into());
    p.push("853".into());
    p.push("389".into());
    p.push("691".into());
    p.push("373".into());
    p.push("258".into());
    p.push("970".into());
    p.push("872".into());
    p.push("262".into());
    p.push("7".into());
    p.push("590".into());
    p.push("290".into());
    p.push("590".into());
    p.push("508".into());
    p.push("239".into());
    p.push("252".into());
    p.push("47".into());
    p.push("963".into());
    p.push("886".into());
    p.push("255".into());
    p.push("670".into());
    p.push("58".into());
    p.push("84".into());
    // normalis number
    let mut startpoint=0;
    if phonenumber[0]==b'0' && phonenumber[1]==b'0' {
        startpoint=2;
    }
    if phonenumber[0]==b'+' {
        startpoint=1;
    }
    // create vec for comparison
    let mut pc3:Vec<u8>= Vec::new();
    pc3.push(phonenumber[startpoint]);
    pc3.push(phonenumber[startpoint+1]);
    pc3.push(phonenumber[startpoint+2]);
    let mut pc2:Vec<u8>= Vec::new();
    pc2.push(phonenumber[startpoint]);
    pc2.push(phonenumber[startpoint+1]);
    let mut pc1:Vec<u8>= Vec::new();
    pc1.push(phonenumber[startpoint]);
    let mut valid=false;
    for xp in p {
        if xp==pc3 || xp==pc2 || xp==pc1 {
            valid =true;
        }
    }
    valid
}
// function to validate a language code
fn aisland_validate_languagecode(language:Vec<u8>) -> bool {
    // check maximum lenght
    if language.len()>2{
        return false;
    }
    // load allowed language code
    let mut p: Vec<Vec<u8>> = Vec::new();
    p.push("aa".into());
    p.push("ab".into());
    p.push("ae".into());
    p.push("af".into());
    p.push("ak".into());
    p.push("am".into());
    p.push("an".into());
    p.push("ar".into());
    p.push("as".into());
    p.push("av".into());
    p.push("ay".into());
    p.push("az".into());
    p.push("ba".into());
    p.push("be".into());
    p.push("bg".into());
    p.push("bh".into());
    p.push("bi".into());
    p.push("bm".into());
    p.push("bn".into());
    p.push("bo".into());
    p.push("br".into());
    p.push("bs".into());
    p.push("ca".into());
    p.push("ce".into());
    p.push("ch".into());
    p.push("co".into());
    p.push("cr".into());
    p.push("cs".into());
    p.push("cu".into());
    p.push("cv".into());
    p.push("cy".into());
    p.push("da".into());
    p.push("de".into());
    p.push("dv".into());
    p.push("dz".into());
    p.push("ee".into());
    p.push("el".into());
    p.push("en".into());
    p.push("eo".into());
    p.push("es".into());
    p.push("et".into());
    p.push("eu".into());
    p.push("fa".into());
    p.push("ff".into());
    p.push("fi".into());
    p.push("fj".into());
    p.push("fo".into());
    p.push("fr".into());
    p.push("fy".into());
    p.push("ga".into());
    p.push("gd".into());
    p.push("gl".into());
    p.push("gn".into());
    p.push("gu".into());
    p.push("gv".into());
    p.push("ha".into());
    p.push("he".into());
    p.push("hi".into());
    p.push("ho".into());
    p.push("hr".into());
    p.push("ht".into());
    p.push("hu".into());
    p.push("hy".into());
    p.push("hz".into());
    p.push("ia".into());
    p.push("id".into());
    p.push("ie".into());
    p.push("ig".into());
    p.push("ii".into());
    p.push("ik".into());
    p.push("io".into());
    p.push("is".into());
    p.push("it".into());
    p.push("iu".into());
    p.push("ja".into());
    p.push("jv".into());
    p.push("ka".into());
    p.push("kg".into());
    p.push("ki".into());
    p.push("kj".into());
    p.push("kk".into());
    p.push("kl".into());
    p.push("km".into());
    p.push("kn".into());
    p.push("ko".into());
    p.push("kr".into());
    p.push("ks".into());
    p.push("ku".into());
    p.push("kv".into());
    p.push("kw".into());
    p.push("ky".into());
    p.push("la".into());
    p.push("lb".into());
    p.push("lg".into());
    p.push("li".into());
    p.push("ln".into());
    p.push("lo".into());
    p.push("lt".into());
    p.push("lu".into());
    p.push("lv".into());
    p.push("mg".into());
    p.push("mh".into());
    p.push("mi".into());
    p.push("mk".into());
    p.push("ml".into());
    p.push("mn".into());
    p.push("mr".into());
    p.push("ms".into());
    p.push("mt".into());
    p.push("my".into());
    p.push("na".into());
    p.push("nb".into());
    p.push("nd".into());
    p.push("ne".into());
    p.push("ng".into());
    p.push("nl".into());
    p.push("nn".into());
    p.push("no".into());
    p.push("nr".into());
    p.push("nv".into());
    p.push("ny".into());
    p.push("oc".into());
    p.push("oj".into());
    p.push("om".into());
    p.push("or".into());
    p.push("os".into());
    p.push("pa".into());
    p.push("pi".into());
    p.push("pl".into());
    p.push("ps".into());
    p.push("pt".into());
    p.push("qu".into());
    p.push("rm".into());
    p.push("rn".into());
    p.push("ro".into());
    p.push("ru".into());
    p.push("rw".into());
    p.push("sa".into());
    p.push("sc".into());
    p.push("sd".into());
    p.push("se".into());
    p.push("sg".into());
    p.push("si".into());
    p.push("sk".into());
    p.push("sl".into());
    p.push("sm".into());
    p.push("sn".into());
    p.push("so".into());
    p.push("sq".into());
    p.push("sr".into());
    p.push("ss".into());
    p.push("st".into());
    p.push("su".into());
    p.push("sv".into());
    p.push("sw".into());
    p.push("ta".into());
    p.push("te".into());
    p.push("tg".into());
    p.push("th".into());
    p.push("ti".into());
    p.push("tk".into());
    p.push("tl".into());
    p.push("tn".into());
    p.push("to".into());
    p.push("tr".into());
    p.push("ts".into());
    p.push("tt".into());
    p.push("tw".into());
    p.push("ty".into());
    p.push("ug".into());
    p.push("uk".into());
    p.push("ur".into());
    p.push("uz".into());
    p.push("ve".into());
    p.push("vi".into());
    p.push("vo".into());
    p.push("wa".into());
    p.push("wo".into());
    p.push("xh".into());
    p.push("yi".into());
    p.push("yo".into());
    p.push("za".into());
    p.push("zh".into());
    p.push("zu".into());
    let mut valid=false;
    for xp in p {
        if xp==language {
            valid =true;
        }
    }
    valid
}
// function to validate the unit measurement system
fn aisland_validate_unitmeasurement(unitmeasurement:Vec<u8>) -> bool {
    // check maximum lenght
    if unitmeasurement.len()>2{
        return false;
    }
    // load allowed language code
    let mut p: Vec<Vec<u8>> = Vec::new();
    p.push("ms".into());  // metric system (Main part of the world)
    p.push("iu".into());  // Imperial System (UK and colonies) 
    p.push("us".into());  // United States customary units (Us and Liberia)
    let mut valid=false;
    for xp in p {
        if xp==unitmeasurement {
            valid =true;
        }
    }
    valid
}
// function to convert vec<u8> to u32
fn vecu8_to_u32(v: Vec<u8>) -> u32 {
    let vslice = v.as_slice();
    let vstr = str::from_utf8(&vslice).unwrap_or("0");
    let vvalue: u32 = u32::from_str(vstr).unwrap_or(0);
    vvalue
}
// function to convert vec<u8> to u128
fn vecu8_to_u128(v: Vec<u8>) -> u128 {
    let vslice = v.as_slice();
    let vstr = str::from_utf8(&vslice).unwrap_or("0");
    let vvalue: u128 = u128::from_str(vstr).unwrap_or(0);
    vvalue
}

