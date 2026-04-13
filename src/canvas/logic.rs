use std::ops;

// Logikai szint (Logic level)
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum LL {
	H, // High
	L, // Low
	U, // Undefined
}

impl LL {
	pub fn to_color(self) -> u32 {
		match self {
			LL::H => 0xff3517F7,
			LL::L => 0xffD4EA41,
			LL::U => 0xffffffff,
		}
	}
}

impl Into<bool> for LL {
	fn into(self) -> bool {
		match self {
			LL::H => true,
			LL::L => false,
			LL::U => unreachable!("Undefined logic level can't be translated to a bool!"),
		}
	}
}

impl From<bool> for LL {
	fn from(value: bool) -> Self {
		match value {
			true => LL::H,
			false => LL::L,
		}
	}
}

impl ops::BitAnd for LL {
	type Output = Self;
	fn bitand(self, rhs: Self) -> Self::Output {
		if self == LL::U || rhs == LL::U {
			LL::U
		} else {
			if self == LL::H && rhs == LL::H {
				LL::H
			} else {
				LL::L
			}
		}
	}
}

impl ops::BitOr for LL {
	type Output = Self;
	fn bitor(self, rhs: Self) -> Self::Output {
		if self == LL::U || rhs == LL::U {
			LL::U
		} else {
			if self == LL::H || rhs == LL::H {
				LL::H
			} else {
				LL::L
			}
		}
	}
}

impl ops::BitXor for LL {
	type Output = Self;
	fn bitxor(self, rhs: Self) -> Self::Output {
		if self == LL::U || rhs == LL::U {
			LL::U
		} else {
			(self != rhs).into()
		}
	}
}


impl ops::BitAndAssign for LL {
	fn bitand_assign(&mut self, rhs: Self) { *self = *self & rhs; }
}

impl ops::BitOrAssign for LL {
	fn bitor_assign(&mut self, rhs: Self) { *self = *self | rhs; }
}

impl ops::BitXorAssign for LL {
	fn bitxor_assign(&mut self, rhs: Self) { *self = *self ^ rhs; }
}

impl ops::Not for LL {
	type Output = Self;
	fn not(self) -> Self::Output {
		match self {
			LL::H => LL::L,
			LL::L => LL::H,
			LL::U => LL::U,
		}
	}
}

impl ops::Neg for LL {
	type Output = Self;
	fn neg(self) -> Self::Output {
		match self {
			LL::H => LL::L,
			LL::L => LL::H,
			LL::U => LL::U,
		}
	}
}
