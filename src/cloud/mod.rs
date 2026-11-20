// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: LicenseRef-Zyvor-Production-1.0

use std::fs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloudProvider {
    None,
    Azure,
    AWS,
    GCP,
    Alibaba,
    Oracle,
    DigitalOcean,
}

impl CloudProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            CloudProvider::None => "none",
            CloudProvider::Azure => "azure",
            CloudProvider::AWS => "aws",
            CloudProvider::GCP => "gcp",
            CloudProvider::Alibaba => "alibaba",
            CloudProvider::Oracle => "oracle",
            CloudProvider::DigitalOcean => "digital ocean",
        }
    }
}

impl std::fmt::Display for CloudProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub fn detect_cloud() -> CloudProvider {
    if detect_azure() {
        CloudProvider::Azure
    } else if detect_ec2() {
        CloudProvider::AWS
    } else if detect_gcp() {
        CloudProvider::GCP
    } else if detect_alibaba() {
        CloudProvider::Alibaba
    } else if detect_oracle() {
        CloudProvider::Oracle
    } else if detect_digital_ocean() {
        CloudProvider::DigitalOcean
    } else {
        CloudProvider::None
    }
}

pub fn detect_azure() -> bool {
    let vendor = fs::read_to_string("/sys/class/dmi/id/sys_vendor").unwrap_or_default();
    let chassis_asset_tag =
        fs::read_to_string("/sys/class/dmi/id/chassis_asset_tag").unwrap_or_default();

    let has_vendor = vendor.contains("Microsoft Corporation");
    let has_chassis_asset_tag = chassis_asset_tag.contains("7783-7084-3265-9085-8269-3286-77");

    has_vendor || has_chassis_asset_tag
}

pub fn detect_ec2() -> bool {
    let hypervisor_uuid = fs::read_to_string("/sys/hypervisor/uuid").unwrap_or_default();
    let product_uuid = fs::read_to_string("/sys/class/dmi/id/product_uuid").unwrap_or_default();
    let product_version =
        fs::read_to_string("/sys/class/dmi/id/product_version").unwrap_or_default();

    hypervisor_uuid.starts_with("ec2")
        || product_uuid.starts_with("ec2")
        || product_version.contains("amazon")
}

pub fn detect_gcp() -> bool {
    let product_name = fs::read_to_string("/sys/class/dmi/id/product_name").unwrap_or_default();
    product_name.contains("Google Compute Engine")
}

pub fn detect_alibaba() -> bool {
    let product_name = fs::read_to_string("/sys/class/dmi/id/product_name").unwrap_or_default();
    product_name.contains("Alibaba Cloud")
}

pub fn detect_digital_ocean() -> bool {
    let vendor = fs::read_to_string("/sys/class/dmi/id/sys_vendor").unwrap_or_default();
    vendor.contains("DigitalOcean")
}

pub fn detect_oracle() -> bool {
    let chassis_asset_tag =
        fs::read_to_string("/sys/class/dmi/id/chassis_asset_tag").unwrap_or_default();
    chassis_asset_tag.contains("OracleCloud")
}
