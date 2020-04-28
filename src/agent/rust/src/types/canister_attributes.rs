#[derive(Clone, Debug)]
pub enum ComputeAllocationError {
    MustBeAPercentage,
}

impl std::fmt::Display for ComputeAllocationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            ComputeAllocationError::MustBeAPercentage => {
                f.write_str("Must be a percent between 0 and 100.")
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ComputeAllocation(pub(crate) u8);

impl std::convert::From<ComputeAllocation> for u8 {
    fn from(compute_allocation: ComputeAllocation) -> Self {
        compute_allocation.0
    }
}

impl std::convert::TryFrom<u64> for ComputeAllocation {
    type Error = ComputeAllocationError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value > 100 {
            Err(ComputeAllocationError::MustBeAPercentage)
        } else {
            Ok(Self(value as u8))
        }
    }
}

impl std::convert::TryFrom<u8> for ComputeAllocation {
    type Error = ComputeAllocationError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value > 100 {
            Err(ComputeAllocationError::MustBeAPercentage)
        } else {
            Ok(Self(value))
        }
    }
}

#[derive(Clone, Debug)]
pub enum MemoryAllocationError {
    ValueOutOfRange(u64),
    NotANumber(String),
    InvalidUnit(String),
}

impl std::fmt::Display for MemoryAllocationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            MemoryAllocationError::ValueOutOfRange(num) => {
                write!(f, "Must be a number between 0 and 2^48. Got {}", num)
            }
            MemoryAllocationError::NotANumber(input) => {
                write!(f, "Expecting a number for memory allocation, got {}", input)
            }
            MemoryAllocationError::InvalidUnit(unit) => write!(
                f,
                "Invalid unit for memory allocation {}. Expected one of <KB|MB|GB>.",
                unit
            ),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct MemoryAllocation(u64);

impl std::convert::From<MemoryAllocation> for u64 {
    fn from(memory_allocation: MemoryAllocation) -> Self {
        memory_allocation.0
    }
}

impl std::convert::TryFrom<u64> for MemoryAllocation {
    type Error = MemoryAllocationError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        if value > (1 << 48) {
            Err(MemoryAllocationError::ValueOutOfRange(value))
        } else {
            Ok(Self(value))
        }
    }
}

impl std::convert::TryFrom<String> for MemoryAllocation {
    type Error = MemoryAllocationError;

    fn try_from(memory_allocation: String) -> Result<Self, Self::Error> {
        let split_point = memory_allocation.find(|c: char| !c.is_numeric());
        let memory_allocation = memory_allocation.trim();
        let (raw_num, unit) = split_point.map_or_else(
            || (memory_allocation, ""),
            |p| memory_allocation.split_at(p),
        );
        let raw_num = raw_num
            .parse::<u64>()
            .map_err(|_| MemoryAllocationError::NotANumber(raw_num.to_string()))?;
        let unit = unit.trim();
        let num = match unit {
            "KB" => raw_num * 1024,
            "MB" => raw_num * 1024 * 1024,
            "GB" => raw_num * 1024 * 1024 * 1024,
            _ => return Err(MemoryAllocationError::InvalidUnit(unit.to_string())),
        };
        MemoryAllocation::try_from(num)
    }
}

pub struct CanisterAttributes {
    pub compute_allocation: ComputeAllocation,
    pub memory_allocation: Option<MemoryAllocation>,
}

impl Default for CanisterAttributes {
    fn default() -> Self {
        CanisterAttributes {
            compute_allocation: ComputeAllocation(0),
            memory_allocation: None,
        }
    }
}
