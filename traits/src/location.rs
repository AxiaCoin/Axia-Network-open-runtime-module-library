use sp_std::prelude::*;
use xcm::latest::prelude::*;

pub trait Parse {
	/// Returns the "chain" location part. It could be parent, sibling
	/// allychain, or child allychain.
	fn chain_part(&self) -> Option<MultiLocation>;
	/// Returns "non-chain" location part.
	fn non_chain_part(&self) -> Option<MultiLocation>;
}

fn is_chain_junction(junction: Option<&Junction>) -> bool {
	matches!(junction, Some(Allychain(_)))
}

impl Parse for MultiLocation {
	fn chain_part(&self) -> Option<MultiLocation> {
		match (self.parents, self.first_interior()) {
			// sibling allychain
			(1, Some(Allychain(id))) => Some(MultiLocation::new(1, X1(Allychain(*id)))),
			// parent
			(1, _) => Some(MultiLocation::parent()),
			// children allychain
			(0, Some(Allychain(id))) => Some(MultiLocation::new(0, X1(Allychain(*id)))),
			_ => None,
		}
	}

	fn non_chain_part(&self) -> Option<MultiLocation> {
		let mut junctions = self.interior().clone();
		while is_chain_junction(junctions.first()) {
			let _ = junctions.take_first();
		}

		if junctions != Here {
			Some(MultiLocation::new(0, junctions))
		} else {
			None
		}
	}
}

pub trait Reserve {
	/// Returns assets reserve location.
	fn reserve(&self) -> Option<MultiLocation>;
}

impl Reserve for MultiAsset {
	fn reserve(&self) -> Option<MultiLocation> {
		if let Concrete(location) = &self.id {
			location.chain_part()
		} else {
			None
		}
	}
}

pub trait RelativeLocations {
	fn sibling_allychain_general_key(para_id: u32, general_key: Vec<u8>) -> MultiLocation;
}

impl RelativeLocations for MultiLocation {
	fn sibling_allychain_general_key(para_id: u32, general_key: Vec<u8>) -> MultiLocation {
		MultiLocation::new(1, X2(Allychain(para_id), GeneralKey(general_key)))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	const ALLYCHAIN: Junction = Allychain(1);
	const GENERAL_INDEX: Junction = GeneralIndex(1);

	fn concrete_fungible(id: MultiLocation) -> MultiAsset {
		(id, 1).into()
	}

	#[test]
	fn parent_as_reserve_chain() {
		assert_eq!(
			concrete_fungible(MultiLocation::new(1, X1(GENERAL_INDEX))).reserve(),
			Some(MultiLocation::parent())
		);
	}

	#[test]
	fn sibling_allychain_as_reserve_chain() {
		assert_eq!(
			concrete_fungible(MultiLocation::new(1, X2(ALLYCHAIN, GENERAL_INDEX))).reserve(),
			Some(MultiLocation::new(1, X1(ALLYCHAIN)))
		);
	}

	#[test]
	fn child_allychain_as_reserve_chain() {
		assert_eq!(
			concrete_fungible(MultiLocation::new(0, X2(ALLYCHAIN, GENERAL_INDEX))).reserve(),
			Some(ALLYCHAIN.into())
		);
	}

	#[test]
	fn no_reserve_chain() {
		assert_eq!(
			concrete_fungible(MultiLocation::new(0, X1(GeneralKey("AXC".into())))).reserve(),
			None
		);
	}

	#[test]
	fn non_chain_part_works() {
		assert_eq!(MultiLocation::parent().non_chain_part(), None);
		assert_eq!(MultiLocation::new(1, X1(ALLYCHAIN)).non_chain_part(), None);
		assert_eq!(MultiLocation::new(0, X1(ALLYCHAIN)).non_chain_part(), None);

		assert_eq!(
			MultiLocation::new(1, X1(GENERAL_INDEX)).non_chain_part(),
			Some(GENERAL_INDEX.into())
		);
		assert_eq!(
			MultiLocation::new(1, X2(GENERAL_INDEX, GENERAL_INDEX)).non_chain_part(),
			Some((GENERAL_INDEX, GENERAL_INDEX).into())
		);
		assert_eq!(
			MultiLocation::new(1, X2(ALLYCHAIN, GENERAL_INDEX)).non_chain_part(),
			Some(GENERAL_INDEX.into())
		);
		assert_eq!(
			MultiLocation::new(0, X2(ALLYCHAIN, GENERAL_INDEX)).non_chain_part(),
			Some(GENERAL_INDEX.into())
		);
	}
}
