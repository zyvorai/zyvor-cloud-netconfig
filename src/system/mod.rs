// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: LicenseRef-Zyvor-Production-1.0

use anyhow::{Context, Result};
use caps::{CapSet, Capability};
use nix::unistd::{setgid, setuid, Gid, Uid, User};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub struct Credential {
    pub uid: Uid,
    pub gid: Gid,
}

pub fn get_user_credentials(username: Option<&str>) -> Result<Credential> {
    let user = match username {
        Some(name) => User::from_name(name)?.context(format!("User '{}' not found", name))?,
        None => User::from_uid(Uid::current())?.context("Current user not found")?,
    };

    Ok(Credential {
        uid: user.uid,
        gid: user.gid,
    })
}

pub fn switch_user(cred: &Credential) -> Result<()> {
    setgid(cred.gid).context("Failed to set GID")?;
    setuid(cred.uid).context("Failed to set UID")?;
    Ok(())
}

pub fn apply_capability(_cred: &Credential) -> Result<()> {
    // Clear all capabilities
    caps::clear(None, CapSet::Permitted)?;
    caps::clear(None, CapSet::Effective)?;
    caps::clear(None, CapSet::Inheritable)?;

    // Set CAP_NET_ADMIN
    let net_admin = HashSet::from([Capability::CAP_NET_ADMIN]);
    caps::set(None, CapSet::Permitted, &net_admin)?;
    caps::set(None, CapSet::Effective, &net_admin)?;
    caps::set(None, CapSet::Inheritable, &net_admin)?;

    // Set ambient capabilities (requires CAP_SETPCAP)
    if caps::has_cap(None, CapSet::Permitted, Capability::CAP_SETPCAP).unwrap_or(false) {
        caps::set(None, CapSet::Ambient, &net_admin)?;
    }

    Ok(())
}

pub fn enable_keep_capability() -> Result<()> {
    unsafe {
        let ret = libc::prctl(libc::PR_SET_KEEPCAPS, 1, 0, 0, 0);
        if ret != 0 {
            return Err(anyhow::anyhow!("Failed to enable keep capabilities"));
        }
    }
    Ok(())
}

pub fn disable_keep_capability() -> Result<()> {
    unsafe {
        let ret = libc::prctl(libc::PR_SET_KEEPCAPS, 0, 0, 0, 0);
        if ret != 0 {
            return Err(anyhow::anyhow!("Failed to disable keep capabilities"));
        }
    }
    Ok(())
}

pub fn create_state_dirs(provider: &str, uid: u32, gid: u32) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    // Create link state directory
    fs::create_dir_all(crate::conf::LINK_STATE_DIR)?;
    let perms = fs::Permissions::from_mode(0o777);
    fs::set_permissions(crate::conf::LINK_STATE_DIR, perms.clone())?;
    chown(crate::conf::LINK_STATE_DIR, uid, gid)?;

    // Change ownership of system state directory
    chown(crate::conf::SYSTEM_STATE_DIR, uid, gid)?;

    // Create provider-specific directory
    let provider_dir = Path::new(crate::conf::SYSTEM_STATE_DIR).join(provider);
    fs::create_dir_all(&provider_dir)?;
    fs::set_permissions(&provider_dir, perms)?;
    chown(provider_dir.to_str().unwrap(), uid, gid)?;

    Ok(())
}

pub fn create_and_save_json<T: serde::Serialize>(path: &str, content: &T) -> Result<()> {
    let json = serde_json::to_string_pretty(content)?;
    fs::write(path, json)?;

    use std::os::unix::fs::PermissionsExt;
    let perms = fs::Permissions::from_mode(0o755);
    fs::set_permissions(path, perms)?;

    Ok(())
}

fn chown(path: &str, uid: u32, gid: u32) -> Result<()> {
    use std::ffi::CString;

    let c_path = CString::new(path)?;
    unsafe {
        let ret = libc::chown(c_path.as_ptr(), uid, gid);
        if ret != 0 {
            return Err(anyhow::anyhow!(
                "Failed to chown {}: {}",
                path,
                std::io::Error::last_os_error()
            ));
        }
    }
    Ok(())
}
