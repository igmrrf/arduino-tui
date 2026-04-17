use serde::Deserialize;
use tokio::process::Command;

#[derive(Debug, Clone)]
pub struct LibraryInfo {
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub sentence: Option<String>,
    pub category: Option<String>,
    pub is_installed: bool,
}

// Search JSON Structures
#[derive(Debug, Deserialize)]
struct SearchRelease {
    author: Option<String>,
    version: String,
    sentence: Option<String>,
    category: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SearchLibrary {
    name: String,
    latest: SearchRelease,
}

#[derive(Debug, Deserialize)]
struct SearchResult {
    libraries: Vec<SearchLibrary>,
}

// List JSON Structures
#[derive(Debug, Deserialize)]
struct ListLibraryData {
    name: String,
    version: String,
    author: Option<String>,
    sentence: Option<String>,
    category: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ListInstalledLibrary {
    library: ListLibraryData,
}

#[derive(Debug, Deserialize)]
struct ListResult {
    installed_libraries: Option<Vec<ListInstalledLibrary>>,
}

pub async fn search_libraries(query: &str) -> Result<Vec<LibraryInfo>, String> {
    let mut cmd = Command::new("arduino-cli");
    cmd.kill_on_drop(true);
    cmd.arg("lib").arg("search");
    if !query.is_empty() {
        cmd.arg(query);
    }
    cmd.arg("--format").arg("json");

    let output = cmd.output().await.map_err(|e| e.to_string())?;
    
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let result: SearchResult = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    
    let libs = result.libraries.into_iter().map(|lib| LibraryInfo {
        name: lib.name,
        version: lib.latest.version,
        author: lib.latest.author,
        sentence: lib.latest.sentence,
        category: lib.latest.category,
        is_installed: false, // We'll determine this by comparing with installed list
    }).collect();

    Ok(libs)
}

pub async fn list_installed_libraries() -> Result<Vec<LibraryInfo>, String> {
    let mut cmd = Command::new("arduino-cli");
    cmd.kill_on_drop(true);
    cmd.arg("lib").arg("list").arg("--format").arg("json");

    let output = cmd.output().await.map_err(|e| e.to_string())?;
    
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let result: ListResult = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;
    
    let installed = result.installed_libraries.unwrap_or_default();
    
    let libs = installed.into_iter().map(|lib| LibraryInfo {
        name: lib.library.name,
        version: lib.library.version,
        author: lib.library.author,
        sentence: lib.library.sentence,
        category: lib.library.category,
        is_installed: true,
    }).collect();

    Ok(libs)
}

pub async fn install_library(name: &str) -> Result<(), String> {
    let mut cmd = Command::new("arduino-cli");
    cmd.kill_on_drop(true);
    cmd.arg("lib").arg("install").arg(name);

    let output = cmd.output().await.map_err(|e| e.to_string())?;
    
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    Ok(())
}

pub async fn uninstall_library(name: &str) -> Result<(), String> {
    let mut cmd = Command::new("arduino-cli");
    cmd.kill_on_drop(true);
    cmd.arg("lib").arg("uninstall").arg(name);

    let output = cmd.output().await.map_err(|e| e.to_string())?;
    
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    Ok(())
}
