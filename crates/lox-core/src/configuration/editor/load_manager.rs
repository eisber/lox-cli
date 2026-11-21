use super::{EditError, legacy::State};
use std::{collections::BTreeMap, error::Error, fmt};
use xmltree::Element;

#[cfg(feature = "configuration-xml")]
mod movement;
#[cfg(feature = "configuration-xml")]
mod name;
#[cfg(feature = "configuration-xml")]
pub use movement::{
    LoadManagerMove, LoadManagerMoveDirection, LoadManagerMoveError, LoadManagerMoveOutcome,
    move_load_manager_load,
};
#[cfg(feature = "configuration-xml")]
pub use name::{
    LoadManagerNameChange, LoadManagerNameError, LoadManagerNameOutcome, set_load_manager_load_name,
};

/// One direct Load record in the selected Load Manager.
#[derive(Debug, PartialEq, Eq)]
pub struct LoadManagerLoad {
    /// One-based XML record order, not a runtime shedding priority.
    pub priority: usize,
    /// Resolved connector association. An absent `ioIdx` means zero.
    pub io_index: u8,
    /// Loader-decoded attributes, including unknown keys and original value
    /// spellings as seen by the loader. No attributes are synthesized.
    pub attributes: BTreeMap<String, String>,
}

/// Owned stored Load Manager inspection. Missing block `U` or `Title` is empty.
#[derive(Debug, PartialEq, Eq)]
pub struct LoadManagerInspection {
    pub uuid: String,
    pub title: String,
    pub loads: Vec<LoadManagerLoad>,
}

/// Failure to read the existing complete Load Manager shape.
#[derive(Debug)]
#[non_exhaustive]
pub enum LoadManagerInspectionError {
    /// The original shared editor load or selection error.
    Edit(EditError),
    NotLoadManager {
        selector: String,
    },
    RecordCount {
        found: usize,
    },
    InvalidIoIndex {
        raw: String,
    },
    DuplicateIoIndex {
        io_index: u8,
    },
}

impl fmt::Display for LoadManagerInspectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Edit(error) => fmt::Display::fmt(error, f),
            Self::NotLoadManager { selector } => write!(
                f,
                "'{selector}' is not a LoadShed (Load Manager). Use `lox config controls FILE --type LoadShed` to find one."
            ),
            Self::RecordCount { found } => write!(
                f,
                "Unsupported Load Manager shape: expected exactly 12 direct Load records, found {found}. Sparse/extended lists are not supported; no defaults were fabricated."
            ),
            Self::InvalidIoIndex { .. } => f.write_str(
                "Unsupported Load Manager shape: ioIdx must be absent (native zero) or decimal digits in 0..11. Explicit empty, signed, whitespace and nonnumeric identities are not supported."
            ),
            Self::DuplicateIoIndex { io_index } => write!(
                f,
                "Unsupported Load Manager shape: duplicate ioIdx {io_index}; identities must be unique."
            ),
        }
    }
}

impl Error for LoadManagerInspectionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Edit(error) => Some(error),
            _ => None,
        }
    }
}

/// Read the stored records of one legacy-selected `C Type="LoadShed"`.
///
/// The legacy selector runs before exact type checking. Exactly twelve direct
/// `Load` records are required, with unique resolved `ioIdx` values in 0..=11.
/// An omitted `ioIdx` resolves to zero, but remains absent from `attributes`.
/// Other attributes (including name, power and timing) are not interpreted,
/// defaulted or validated. This is not a native validator or a runtime report.
/// The shared loader's lox-n37w quoted-text/digit-name limitation is unchanged.
/// No editing, serialization or filesystem I/O is performed.
pub fn inspect_load_manager(
    bytes: &[u8],
    selector: &str,
) -> Result<LoadManagerInspection, LoadManagerInspectionError> {
    State::load(bytes)
        .map_err(LoadManagerInspectionError::Edit)?
        .load_manager_inspection_selected(selector)
}

impl State {
    #[doc(hidden)]
    pub fn load_manager_path(
        &self,
        selector: &str,
    ) -> Result<Vec<usize>, LoadManagerInspectionError> {
        let path = self
            .require_one(selector)
            .map_err(|error| LoadManagerInspectionError::Edit(error.into()))?;
        let block = self.get_element(&path);
        if block.name != "C" || block.attributes.get("Type").map(String::as_str) != Some("LoadShed")
        {
            return Err(LoadManagerInspectionError::NotLoadManager {
                selector: selector.to_owned(),
            });
        }
        Ok(path)
    }

    #[doc(hidden)]
    pub fn load_manager_inspection_selected(
        &self,
        selector: &str,
    ) -> Result<LoadManagerInspection, LoadManagerInspectionError> {
        let path = self.load_manager_path(selector)?;
        let block = self.get_element(&path);
        Ok(inspect_positions(block, &load_positions(block)?))
    }
}

fn inspect_positions(block: &Element, positions: &[usize]) -> LoadManagerInspection {
    let loads = positions
        .iter()
        .enumerate()
        .map(|(index, &position)| {
            let load = block.children[position].as_element().unwrap();
            LoadManagerLoad {
                priority: index + 1,
                io_index: resolve_io_index(load).unwrap(),
                attributes: load
                    .attributes
                    .iter()
                    .map(|(key, value)| (key.clone(), value.clone()))
                    .collect(),
            }
        })
        .collect();
    LoadManagerInspection {
        uuid: block.attributes.get("U").cloned().unwrap_or_default(),
        title: block.attributes.get("Title").cloned().unwrap_or_default(),
        loads,
    }
}

pub fn load_positions(block: &Element) -> Result<Vec<usize>, LoadManagerInspectionError> {
    let positions: Vec<_> = block
        .children
        .iter()
        .enumerate()
        .filter_map(|(index, node)| {
            node.as_element()
                .filter(|element| element.name == "Load")
                .map(|_| index)
        })
        .collect();
    if positions.len() != 12 {
        return Err(LoadManagerInspectionError::RecordCount {
            found: positions.len(),
        });
    }
    let mut seen = [false; 12];
    for position in &positions {
        let load = block.children[*position].as_element().unwrap();
        let io_index = resolve_io_index(load)?;
        if seen[usize::from(io_index)] {
            return Err(LoadManagerInspectionError::DuplicateIoIndex { io_index });
        }
        seen[usize::from(io_index)] = true;
    }
    Ok(positions)
}

pub fn resolve_io_index(load: &Element) -> Result<u8, LoadManagerInspectionError> {
    // The native serializer omits zero; an absent attribute reads back as zero.
    let Some(raw) = load.attributes.get("ioIdx") else {
        return Ok(0);
    };
    if !raw.is_empty()
        && raw.bytes().all(|byte| byte.is_ascii_digit())
        && let Ok(index @ 0..=11) = raw.parse::<u8>()
    {
        return Ok(index);
    }
    Err(LoadManagerInspectionError::InvalidIoIndex { raw: raw.clone() })
}
