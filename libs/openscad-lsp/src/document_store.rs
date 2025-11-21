use std::collections::HashMap;
use tower_lsp::lsp_types::Url;

#[derive(Debug, Default)]
pub struct DocumentStore {
    documents: HashMap<Url, String>,
}

impl DocumentStore {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
        }
    }

    pub fn update(&mut self, uri: Url, text: String) {
        self.documents.insert(uri, text);
    }

    pub fn get(&self, uri: &Url) -> Option<&String> {
        self.documents.get(uri)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_store() {
        let mut store = DocumentStore::new();
        let uri = Url::parse("file:///test.scad").unwrap();
        let text = "cube([10, 10, 10]);".to_string();

        store.update(uri.clone(), text.clone());
        assert_eq!(store.get(&uri), Some(&text));

        let new_text = "sphere(10);".to_string();
        store.update(uri.clone(), new_text.clone());
        assert_eq!(store.get(&uri), Some(&new_text));
    }
}
