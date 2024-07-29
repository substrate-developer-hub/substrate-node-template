### Global settings
```rust
DefaultTake: u16 = 11_796; // 18% honest number.
TxRateLimit: u64 = 1; // [1 @ 64,888]
```

### netuid 1 (text_prompting)
```rust
Rho: u16 = 10;
Kappa: u16 = 32_767; // 0.5 = 65535/2 
MaxAllowedUids: u16 = 1024;
Issuance: u64 = 0;
MinAllowedWeights: u16 = 8;
EmissionValue: u64 = 142_223_000;
MaxWeightsLimit: 455; // 455/2^16 = 0.0069
ValidatorBatchSize: u16 = 1;
ValidatorSequenceLen: u16 = 2048; // 2048
ValidatorEpochLen: u16 = 100;
ValidatorEpochsPerReset: u16 = 60;
ValidatorExcludeQuantile: u16 = 0; // 0% of u16
ValidatorPruneLen: u64 = 1;
ValidatorLogitsDivergence: u16 = 1310; // 2% of u16
ScalingLawPower: u16 = 50; // 0.5
SynergyScalingLawPower: u16 = 50; // 0.5
MaxAllowedValidators: u16 = 128;
Tempo: u16 = 99;
Difficulty: u64 = 10_000_000;
AdjustmentInterval: u16 = 225;
TargetRegistrationsPerInterval: u16 = 2;
ImmunityPeriod: u16 = 7200;
ActivityCutoff: u16 = 5000;
MaxRegistrationsPerBlock: u16 = 1;
PruningScore : u16 = u16::MAX;
BondsMovingAverage: u64 = 900_000;
WeightsVersionKey: u64 = 1020;
MinDifficulty: u64 = 10_000_000;
MaxDifficulty: u64 = u64::MAX / 4;
ServingRateLimit: u64 = 10; 
Burn: u64 = 1_000_000_000; // 1 tao
MinBurn: u64 = 1_000_000_000; // 1 tao
MaxBurn: u64 = 100_000_000_000; // 100 tao
WeightsSetRateLimit: u64 = 100;
```

### netuid 3 (causallmnext)
```rust
Rho: u16 = 10;
Kappa: u16 = 32_767; // 0.5 = 65535/2 
MaxAllowedUids: u16 = 4096;
Issuance: u64 = 0;
MinAllowedWeights: u16 = 50;
EmissionValue: u64 = 857_777_000;
MaxWeightsLimit: u16 = 655; // 655/2^16 = 0.01 [655 @ 7,160]
ValidatorBatchSize: u16 = 32; // 32
ValidatorSequenceLen: u16 = 256; // 256
ValidatorEpochLen: u16 = 250; // [250 @ 7,161]
ValidatorEpochsPerReset: u16 = 60;
ValidatorExcludeQuantile: u16 = 3277; // 5% of u16 [3277 @ 65,065]
ValidatorPruneLen: u64 = 1;
ValidatorLogitsDivergence: u16 = 1310; // 2% of u16
ScalingLawPower: u16 = 50; // 0.5
SynergyScalingLawPower: u16 = 50; // 0.5
MaxAllowedValidators: u16 = 128;
Tempo: u16 = 99;
Difficulty: u64 = 671_088_640_000_000; // Same as nakamoto at block = 3606775 [671T @ 26,310]
AdjustmentInterval: u16 = 100;
TargetRegistrationsPerInterval: u16 = 2;
ImmunityPeriod: u16 = 4096;
ActivityCutoff: u16 = 5000; // [5000 @ 7,163]
MaxRegistrationsPerBlock: u16 = 1;
PruningScore : u16 = u16::MAX;
BondsMovingAverage: u64 = 900_000;
WeightsVersionKey: u64 = 400;
MinDifficulty: u64 = 10_000_000;
MaxDifficulty: u64 = u64::MAX / 4;
ServingRateLimit: u64 = 250; // [250 @ 7,166]
Burn: u64 = 100_000_000_000; // 100 tao [100 tao @ 26,310]
MinBurn: u64 = 1_000_000_000; // 1 tao [1 tao @ 26,310]
MaxBurn: u64 = 21_000_000_000_000_000; // 21M tao [21M tao @ 26,310]
WeightsSetRateLimit: u64 = 250; // [250 @ 7,168]
```
