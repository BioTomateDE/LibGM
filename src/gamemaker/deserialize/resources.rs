use crate::gamemaker::deserialize::reader::DataReader;
use crate::gamemaker::elements::texture_page_items::GMTexturePageItem;
use crate::prelude::*;
use crate::util::fmt::typename;
use std::collections::HashMap;

/// GMRef has (fake) generic types to make it clearer which type it belongs to (`name: GMRef` vs `name: GMRef<String>`).
/// It can be resolved to the data it references using the `.resolve()` method, which needs the list the elements are stored in.
/// This means that removing or inserting elements in the middle of the list will shift all their `GMRef`s; breaking them.
#[derive(Hash, PartialEq, Eq)]
pub struct GMRef<T> {
    pub index: u32,
    // Marker needs to be here to ignore "unused generic T" error; doesn't store any data
    _marker: std::marker::PhantomData<T>,
}

impl<T> GMRef<T> {
    /// Creates a new GameMaker reference with the specified index.
    /// The fake generic type can often be omitted (if the compiler can infer it).
    pub fn new(index: u32) -> GMRef<T> {
        Self { index, _marker: std::marker::PhantomData }
    }
}

impl<T> Clone for GMRef<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for GMRef<T> {}
impl<T> std::fmt::Debug for GMRef<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GMRef<{}#{}>", typename::<T>(), self.index)
    }
}

impl<'a, T> GMRef<T> {
    /// Attempts to resolve this reference to an element in the given list by its index.
    ///
    /// Returns a reference to the element if the index is valid, or an error string if out of bounds.
    ///
    /// # Parameters
    /// - `elements_by_index`: A vector of elements indexed by `self.index`.
    ///
    /// # Errors
    /// Returns an error if `self.index` is out of bounds for the provided vector.
    ///
    pub fn resolve(&self, elements_by_index: &'a Vec<T>) -> Result<&'a T> {
        elements_by_index.get(self.index as usize).with_context(|| {
            format!(
                "Could not resolve {} reference with index {} in list with length {}",
                typename::<T>(),
                self.index,
                elements_by_index.len(),
            )
        })
    }
}

impl DataReader<'_> {
    /// Read a standard GameMaker string reference.
    pub fn read_gm_string(&mut self) -> Result<GMRef<String>> {
        let occurrence_position = self.read_u32()?;
        self.resolve_occurrence(occurrence_position, &self.string_occurrences)
    }

    pub fn read_gm_texture(&mut self) -> Result<GMRef<GMTexturePageItem>> {
        let occurrence_position = self.read_u32()?;
        self.resolve_occurrence(occurrence_position, &self.texture_page_item_occurrences)
    }

    pub fn read_gm_string_opt(&mut self) -> Result<Option<GMRef<String>>> {
        let occurrence_position = self.read_u32()?;
        if occurrence_position == 0 {
            return Ok(None);
        }
        Ok(Some(
            self.resolve_occurrence(occurrence_position, &self.string_occurrences)?,
        ))
    }

    pub fn read_gm_texture_opt(&mut self) -> Result<Option<GMRef<GMTexturePageItem>>> {
        let occurrence_position = self.read_u32()?;
        if occurrence_position == 0 {
            return Ok(None);
        }
        Ok(Some(self.resolve_occurrence(
            occurrence_position,
            &self.texture_page_item_occurrences,
        )?))
    }

    fn resolve_occurrence<T>(
        &self,
        occurrence_position: u32,
        occurrence_map: &HashMap<u32, GMRef<T>>,
    ) -> Result<GMRef<T>> {
        match occurrence_map.get(&occurrence_position) {
            Some(gm_ref) => Ok(*gm_ref),
            None => bail!(
                "Could not read {} with occurrence position {} at pointer position {} \
                because it doesn't exist in the occurrence map with {} items",
                typename::<T>(),
                occurrence_position,
                self.cur_pos - 4,
                occurrence_map.len(),
            ),
        }
    }

    pub fn resolve_gm_str(&self, string_ref: GMRef<String>) -> Result<&String> {
        string_ref.resolve(&self.strings)
    }

    pub fn display_gm_str(&self, string_ref: GMRef<String>) -> &str {
        string_ref.display(&self.strings)
    }

    pub fn read_resource_by_id<T>(&mut self) -> Result<GMRef<T>> {
        const CTX: &str = "reading resource by ID";
        let number = self.read_u32().context(CTX)?;
        self.check_resource_limit(number).context(CTX)?;
        Ok(GMRef::new(number))
    }

    pub fn read_resource_by_id_opt<T>(&mut self) -> Result<Option<GMRef<T>>> {
        let number = self.read_i32()?;
        self.resource_opt_from_i32(number)
    }

    pub fn resource_opt_from_i32<T>(&mut self, number: i32) -> Result<Option<GMRef<T>>> {
        const CTX: &str = "parsing optional resource by ID";
        if number == -1 {
            return Ok(None);
        }
        let number: u32 = number
            .try_into()
            .ok()
            .with_context(|| format!("Invalid negative number {number} (0x{number:08X})"))
            .context(CTX)?;
        self.check_resource_limit(number).context(CTX)?;
        Ok(Some(GMRef::new(number)))
    }

    fn check_resource_limit(&self, number: u32) -> Result<()> {
        // Increase limit if not enough
        const FAILSAFE_COUNT: u32 = 500_000;
        integrity_assert! {
            number < FAILSAFE_COUNT,
            "Number {number} exceeds failsafe limit of {FAILSAFE_COUNT}"
        }
        Ok(())
    }
}
