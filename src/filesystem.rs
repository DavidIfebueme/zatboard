use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileType {
    Directory,
    File,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    pub name: String,
    pub file_type: FileType,
    pub content: Option<String>,
    pub children: HashMap<String, FileNode>,
    pub permissions: Permissions,
    pub created_by: String,
    pub created_at: u64,
    pub modified_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permissions {
    pub owner: String,
    pub read_users: Vec<String>,
    pub write_users: Vec<String>,
    pub public_read: bool,
    pub public_write: bool,
}

impl Permissions {
    pub fn new(owner: String) -> Self {
        Permissions {
            owner: owner.clone(),
            read_users: vec![owner.clone()],
            write_users: vec![owner],
            public_read: true,
            public_write: false,
        }
    }
    
    pub fn can_read(&self, user: &str) -> bool {
        self.public_read || 
        self.owner == user || 
        self.read_users.contains(&user.to_string())
    }
    
    pub fn can_write(&self, user: &str) -> bool {
        self.public_write || 
        self.owner == user || 
        self.write_users.contains(&user.to_string())
    }
    
    pub fn add_read_permission(&mut self, user: String) {
        if !self.read_users.contains(&user) {
            self.read_users.push(user);
        }
    }
    
    pub fn add_write_permission(&mut self, user: String) {
        if !self.write_users.contains(&user) {
            self.write_users.push(user);
        }
    }
}

impl FileNode {
    pub fn new_directory(name: String, owner: String) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        FileNode {
            name,
            file_type: FileType::Directory,
            content: None,
            children: HashMap::new(),
            permissions: Permissions::new(owner.clone()),
            created_by: owner,
            created_at: now,
            modified_at: now,
        }
    }
    
    pub fn new_file(name: String, content: String, owner: String) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        FileNode {
            name,
            file_type: FileType::File,
            content: Some(content),
            children: HashMap::new(),
            permissions: Permissions::new(owner.clone()),
            created_by: owner,
            created_at: now,
            modified_at: now,
        }
    }
    
    pub fn add_child(&mut self, child: FileNode) -> Result<(), String> {
        if self.file_type != FileType::Directory {
            return Err("Cannot add children to a file".to_string());
        }
        
        self.children.insert(child.name.clone(), child);
        self.modified_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Ok(())
    }
    
    pub fn get_child(&self, name: &str) -> Option<&FileNode> {
        self.children.get(name)
    }
    
    pub fn get_child_mut(&mut self, name: &str) -> Option<&mut FileNode> {
        self.children.get_mut(name)
    }
    
    pub fn list_children(&self) -> Vec<String> {
        let mut items: Vec<String> = self.children.keys()
            .map(|name| {
                let child = &self.children[name];
                match child.file_type {
                    FileType::Directory => format!("{}/", name),
                    FileType::File => name.clone(),
                }
            })
            .collect();
        items.sort();
        items
    }
    
    pub fn update_content(&mut self, content: String) -> Result<(), String> {
        if self.file_type != FileType::File {
            return Err("Cannot set content on a directory".to_string());
        }
        
        self.content = Some(content);
        self.modified_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Ok(())
    }
}

#[derive(Debug)]
pub struct FileSystem {
    pub root: FileNode,
}

impl FileSystem {
    pub fn new(owner: String) -> Self {
        FileSystem {
            root: FileNode::new_directory("/".to_string(), owner),
        }
    }
    
    pub fn resolve_path(&self, path: &str) -> Option<&FileNode> {
        if path == "/" {
            return Some(&self.root);
        }
        
        let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
        let mut current = &self.root;
        
        for part in parts {
            if part.is_empty() {
                continue;
            }
            current = current.get_child(part)?;
        }
        
        Some(current)
    }
    
    pub fn resolve_path_mut(&mut self, path: &str) -> Option<&mut FileNode> {
        if path == "/" {
            return Some(&mut self.root);
        }
        
        let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
        let mut current = &mut self.root;
        
        for part in parts {
            if part.is_empty() {
                continue;
            }
            current = current.get_child_mut(part)?;
        }
        
        Some(current)
    }
    
    pub fn create_directory(&mut self, path: &str, owner: String) -> Result<(), String> {
        let (parent_path, dir_name) = self.split_path(path)?;
        
        let parent = self.resolve_path_mut(&parent_path)
            .ok_or_else(|| format!("Parent directory not found: {}", parent_path))?;
            
        if !parent.permissions.can_write(&owner) {
            return Err("Permission denied: cannot write to parent directory".to_string());
        }
        
        if parent.children.contains_key(&dir_name) {
            return Err("Directory already exists".to_string());
        }
        
        let new_dir = FileNode::new_directory(dir_name.clone(), owner);
        parent.add_child(new_dir)?;
        
        Ok(())
    }
    
    pub fn create_file(&mut self, path: &str, content: String, owner: String) -> Result<(), String> {
        let (parent_path, file_name) = self.split_path(path)?;
        
        let parent = self.resolve_path_mut(&parent_path)
            .ok_or_else(|| format!("Parent directory not found: {}", parent_path))?;
            
        if !parent.permissions.can_write(&owner) {
            return Err("Permission denied: cannot write to parent directory".to_string());
        }
        
        let new_file = FileNode::new_file(file_name.clone(), content, owner);
        parent.add_child(new_file)?;
        
        Ok(())
    }
    
    fn split_path(&self, path: &str) -> Result<(String, String), String> {
        let path = path.trim_end_matches('/');
        if path == "/" {
            return Err("Cannot create root directory".to_string());
        }
        
        let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
        if parts.is_empty() || parts[parts.len() - 1].is_empty() {
            return Err("Invalid path".to_string());
        }
        
        let file_name = parts[parts.len() - 1].to_string();
        let parent_path = if parts.len() == 1 {
            "/".to_string()
        } else {
            "/".to_string() + &parts[0..parts.len() - 1].join("/")
        };
        
        Ok((parent_path, file_name))
    }

    pub fn remove(&mut self, path: &str, user: &str) -> Result<(), String> {
        if path == "/" {
            return Err("Cannot remove root directory".to_string());
        }
        
        let (parent_path, item_name) = self.split_path(path)?;
        
        let parent = self.resolve_path_mut(&parent_path)
            .ok_or_else(|| format!("Parent directory not found: {}", parent_path))?;
            
        if !parent.permissions.can_write(user) {
            return Err("Permission denied: cannot write to parent directory".to_string());
        }
        
        if !parent.children.contains_key(&item_name) {
            return Err(format!("File or directory not found: {}", path));
        }
        
        let item = parent.children.get(&item_name).unwrap();
        if item.permissions.owner != user && !parent.permissions.can_write(user) {
            return Err("Permission denied: cannot remove item".to_string());
        }
        
        parent.children.remove(&item_name);
        parent.modified_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_filesystem_creation() {
        let fs = FileSystem::new("zs1owner123".to_string());
        assert_eq!(fs.root.name, "/");
        assert_eq!(fs.root.file_type, FileType::Directory);
    }
    
    #[test]
    fn test_directory_creation() {
        let mut fs = FileSystem::new("zs1owner123".to_string());
        
        let result = fs.create_directory("/home", "zs1owner123".to_string());
        assert!(result.is_ok());
        
        let home_dir = fs.resolve_path("/home");
        assert!(home_dir.is_some());
        assert_eq!(home_dir.unwrap().file_type, FileType::Directory);
    }
    
    #[test]
    fn test_file_creation() {
        let mut fs = FileSystem::new("zs1owner123".to_string());
        
        fs.create_directory("/home", "zs1owner123".to_string()).unwrap();
        let result = fs.create_file("/home/readme.txt", "Hello World!".to_string(), "zs1owner123".to_string());
        assert!(result.is_ok());
        
        let file = fs.resolve_path("/home/readme.txt");
        assert!(file.is_some());
        assert_eq!(file.unwrap().content, Some("Hello World!".to_string()));
    }
    
    #[test]
    fn test_permissions() {
        let perms = Permissions::new("zs1owner123".to_string());
        
        assert!(perms.can_read("zs1owner123"));
        assert!(perms.can_write("zs1owner123"));
        assert!(perms.can_read("zs1other456"));
        assert!(!perms.can_write("zs1other456"));
    }
    
    #[test]
    fn test_directory_listing() {
        let mut fs = FileSystem::new("zs1owner123".to_string());
        
        fs.create_directory("/home", "zs1owner123".to_string()).unwrap();
        fs.create_file("/home/file1.txt", "content1".to_string(), "zs1owner123".to_string()).unwrap();
        fs.create_file("/home/file2.txt", "content2".to_string(), "zs1owner123".to_string()).unwrap();
        
        let home_dir = fs.resolve_path("/home").unwrap();
        let listing = home_dir.list_children();
        
        assert_eq!(listing, vec!["file1.txt", "file2.txt"]);
    }

    #[test]
    fn test_file_removal() {
        let mut fs = FileSystem::new("zs1owner123".to_string());
        
        fs.create_directory("/home", "zs1owner123".to_string()).unwrap();
        fs.create_file("/home/temp.txt", "temporary".to_string(), "zs1owner123".to_string()).unwrap();
        
        assert!(fs.resolve_path("/home/temp.txt").is_some());
        
        let result = fs.remove("/home/temp.txt", "zs1owner123");
        assert!(result.is_ok());
        assert!(fs.resolve_path("/home/temp.txt").is_none());
    }
    
    #[test]
    fn test_directory_removal() {
        let mut fs = FileSystem::new("zs1owner123".to_string());
        
        fs.create_directory("/temp_dir", "zs1owner123".to_string()).unwrap();
        
        assert!(fs.resolve_path("/temp_dir").is_some());
        
        let result = fs.remove("/temp_dir", "zs1owner123");
        assert!(result.is_ok());
        assert!(fs.resolve_path("/temp_dir").is_none());
    }
    
    #[test]
    fn test_remove_permission_denied() {
        let mut fs = FileSystem::new("zs1owner123".to_string());
        
        fs.create_file("/protected.txt", "secret".to_string(), "zs1owner123".to_string()).unwrap();
        
        let result = fs.remove("/protected.txt", "zs1other456");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Permission denied"));
    }
    
    #[test]
    fn test_remove_root_denied() {
        let mut fs = FileSystem::new("zs1owner123".to_string());
        
        let result = fs.remove("/", "zs1owner123");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannot remove root directory"));
    }

}