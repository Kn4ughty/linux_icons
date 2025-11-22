use crate::theme::DirectoryRef;
use crate::{IconFile, Theme};
use qp_trie::wrapper::BString;
use std::sync::Arc;

pub struct ThemeCache {
    theme: Arc<Theme>,
    // Cache of directory names to an Option indicating:
    // - Some(base_dir): the icon exists in this directory, in base_dir.
    // - None: the icon doesn't exist in this directory
    cache: qp_trie::Trie<BString, Vec<(DirectoryRef, IconFile)>>,
}

impl ThemeCache {
    pub fn from_theme(theme: Arc<Theme>) -> Self {
        theme.into()
    }

    /// Find an icon in this theme only, utilizing and populating the internal cache where possible.
    ///
    /// This function is analogous to [Theme::find_icon_here].
    // for people editing this function: make sure to check, and keep in sync, the behaviour of
    // Theme::find_icon_here with this function.
    pub fn find_icon_here(&mut self, icon_name: &str, size: u32, scale: u32) -> Option<IconFile> {
        // If `icon_name` isn't in the cache yet,
        // let's start by finding all(!) of its files; this is more expensive than the normal
        // lookup function, but we pay the cost upfront to make subsequent lookups quicker!

        let icon_files: &Vec<_> = self
            .cache
            .entry(icon_name.into())
            // if this icon isn't in the cache already, find its files and insert those:
            .or_insert_with(|| self.theme.find_icon_files(icon_name).collect());

        // find an exact match:
        for (dir, ico) in icon_files {
            let dir = &self.theme.info.index.directories[*dir];

            if dir.matches_size(size, scale) {
                return Some(ico.clone());
            }
        }

        // else, find the closest match:
        let icon = icon_files.iter().min_by_key(|(dir, _)| {
            let dir = &self.theme.info.index.directories[*dir];

            dir.size_distance(size, scale)
        });

        icon.map(|(_, ico)| ico.clone())
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl From<Arc<Theme>> for ThemeCache {
    fn from(theme: Arc<Theme>) -> Self {
        Self {
            theme,
            cache: Default::default()
        }
    }
}

#[cfg(test)]
mod test {
    use crate::cache::ThemeCache;
    use crate::search::test::test_search;

    #[test]
    fn test_cached_entry_persists() {
        let icons = test_search().search().icons();
        let theme = icons.theme("TestTheme").unwrap();

        let icon_original = theme.find_icon_here("happy", 16, 1).unwrap();

        let mut theme_cache: ThemeCache = theme.into();

        assert!(theme_cache.cache.is_empty(), "cache is not yet populated");

        let icon = theme_cache.find_icon_here("happy", 16, 1).unwrap();
        assert_eq!(icon.icon_name(), "happy");
        println!("{:?}", icon);

        assert!(theme_cache.cache.contains_key_str("happy"), "cache contains happy icon");

        let icon_cached = theme_cache.find_icon_here("happy", 16, 1).unwrap();

        assert_eq!(icon, icon_cached, "cached icon is the same as the first one");
        assert_eq!(icon_original, icon, "cached icon is the same as the original");
    }
}