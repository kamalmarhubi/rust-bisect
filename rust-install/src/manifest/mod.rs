//! Maintains a Rust installation by installing individual Rust
//! platform components from a distribution server.

use rust_manifest::{Component, Manifest, Config, TargettedPackage};
use component::{Components, Transaction, TarGzPackage, Package};
use temp;
use errors::*;
use utils;
use install::InstallPrefix;
use openssl::crypto::hash::{Type, Hasher};
use itertools::Itertools;

pub const DIST_MANIFEST: &'static str = "multirust-dist.toml";
pub const CONFIG_FILE: &'static str = "multirust-config.toml";

#[derive(Debug)]
pub struct Manifestation {
    installation: Components,
    target_triple: String
}

#[derive(Debug)]
pub struct Changes {
    pub add_extensions: Vec<Component>,
    pub remove_extensions: Vec<Component>,
}

impl Changes {
    pub fn none() -> Self {
        Changes {
            add_extensions: Vec::new(),
            remove_extensions: Vec::new(),
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum UpdateStatus { Changed, Unchanged }

impl Manifestation {
    /// Open the install prefix for updates from a distribution
    /// channel.  The install prefix directory does not need to exist;
    /// it will be created as needed. If there's an existing install
    /// then the rust-install installation format will be verified. A
    /// bad installer version is the only reason this will fail.
    pub fn open(prefix: InstallPrefix, triple: &str) -> Result<Self> {
        // TODO: validate the triple with the existing install as well
        // as the metadata format of the existing install
        Ok(Manifestation {
            installation: try!(Components::open(prefix)),
            target_triple: triple.to_string(),
        })
    }

    /// Install or update from a given channel manifest, while
    /// selecting extension components to add or remove.
    ///
    /// `update` takes a manifest describing a release of Rust (which
    /// may be either a freshly-downloaded one, or the same one used
    /// for the previous install), as well as lists off extension
    /// components to add and remove.

    /// From that it schedules a list of components to uninstall and
    /// to uninstall to bring the installation up to date.  It
    /// downloads the components' packages. Then in a Transaction
    /// uninstalls old packages and installs new packages, writes the
    /// distribution manifest to "rustlib/rustup-dist.toml" and a
    /// configuration containing the component name-target pairs to
    /// "rustlib/rustup-config.toml".
    pub fn update(&self,
                  new_manifest: &Manifest,
                  changes: Changes,
                  temp_cfg: &temp::Cfg,
                  notify_handler: NotifyHandler) -> Result<UpdateStatus> {

        // Some vars we're going to need a few times
        let prefix = self.installation.prefix();
        let ref rel_config_path = prefix.rel_manifest_file(CONFIG_FILE);
        let ref config_path = prefix.path().join(rel_config_path);
        let ref rel_installed_manifest_path = prefix.rel_manifest_file(DIST_MANIFEST);
        let ref installed_manifest_path = prefix.path().join(rel_installed_manifest_path);
        let rust_package = try!(new_manifest.get_package("rust"));
        let rust_target_package = try!(rust_package.get_target(&self.target_triple));

        // Load the previous dist manifest
        let ref old_manifest_path = prefix.manifest_file(DIST_MANIFEST);
        let ref old_manifest = if utils::path_exists(old_manifest_path) {
            let ref manifest_str = try!(utils::read_file("installed manifest", old_manifest_path));
            Some(try!(Manifest::parse(manifest_str)))
        } else {
            None
        };

        // Load the configuration and list of installed components.
        let ref config = if utils::path_exists(config_path) {
            let ref config_str = try!(utils::read_file("dist config", config_path));
            Some(try!(Config::parse(config_str)))
        } else {
            None
        };

        // Create the lists of components needed for installation
        let component_lists = try!(build_update_component_lists(new_manifest, old_manifest, config,
                                                                changes, &rust_target_package));
        let (components_to_uninstall,
             components_to_install,
             final_component_list) = component_lists;

        if components_to_uninstall.is_empty() && components_to_install.is_empty() {
            return Ok(UpdateStatus::Unchanged);
        }

        // Map components to urls and hashes
        let mut components_urls_and_hashes: Vec<(Component, String, String)> = Vec::new();
        for component in components_to_install {
            let package = try!(new_manifest.get_package(&component.pkg));
            let target_package = try!(package.get_target(&component.target));
            let c_u_h = (component, target_package.url.clone(), target_package.hash.clone());
            components_urls_and_hashes.push(c_u_h);
        }

        // Download component packages and validate hashes
        let mut things_to_install: Vec<(Component, temp::File)> = Vec::new();
        for (component, url, hash) in components_urls_and_hashes {
            // Download each package to temp file
            let temp_file = try!(temp_cfg.new_file());
            let url_url = try!(utils::parse_url(&url));

            let mut hasher = Hasher::new(Type::SHA256);
            try!(utils::download_file(url_url, &temp_file, Some(&mut hasher), ntfy!(&notify_handler))
                 .map_err(|e| Error::ComponentDownloadFailed(component.clone(), e)));

            let actual_hash = hasher.finish()
                                    .iter()
                                    .map(|b| format!("{:02x}", b))
                                    .join("");

            if hash != actual_hash {
                // Incorrect hash
                return Err(Error::ChecksumFailed {
                    url: url,
                    expected: hash,
                    calculated: actual_hash,
                });
            } else {
                notify_handler.call(Notification::ChecksumValid(&url));
            }

            things_to_install.push((component, temp_file));
        }

        // Begin transaction
        let mut tx = Transaction::new(prefix.clone(), temp_cfg, notify_handler);

        // Uninstall components
        for component in components_to_uninstall {
            tx = try!(self.uninstall_component(&component, tx, notify_handler.clone()));
        }

        // Install components
        for (component, installer_file) in things_to_install {
            let package = try!(TarGzPackage::new_file(&installer_file, temp_cfg));

            // For historical reasons, the rust-installer component
            // names are not the same as the dist manifest component
            // names. Some are just the component name some are the
            // component name plus the target triple.
            let ref name = format!("{}-{}", component.pkg, component.target);
            let ref short_name = format!("{}", component.pkg);

            // If the package doesn't contain the component that the
            // manifest says it does the somebody must be playing a joke on us.
            if !package.contains(name, Some(short_name)) {
                return Err(Error::CorruptComponent(component.pkg.clone()));
            }

            tx = try!(package.install(&self.installation,
                                      name, Some(short_name),
                                      tx));
        }

        // Install new distribution manifest
        let ref new_manifest_str = new_manifest.clone().stringify();
        try!(tx.modify_file(rel_installed_manifest_path.to_owned()));
        try!(utils::write_file("manifest", installed_manifest_path, new_manifest_str));

        // Write configuration.
        //
        // NB: This configuration is mostly for keeping track of the name/target pairs
        // that identify installed components. The rust-installer metadata maintained by
        // `Components` *also* tracks what is installed, but it only tracks names, not
        // name/target. Needs to be fixed in rust-installer.
        let mut config = Config::new();
        config.components = final_component_list;
        let ref config_str = config.stringify();
        try!(tx.modify_file(rel_config_path.to_owned()));
        try!(utils::write_file("dist config", config_path, config_str));

        // End transaction
        tx.commit();

        Ok(UpdateStatus::Changed)
    }

    pub fn uninstall(&self, temp_cfg: &temp::Cfg, notify_handler: NotifyHandler) -> Result<()> {
        let prefix = self.installation.prefix();

        let mut tx = Transaction::new(prefix.clone(), temp_cfg, notify_handler);

        // Read configuration and delete it
        let rel_config_path = prefix.rel_manifest_file(CONFIG_FILE);
        let ref config_str = try!(utils::read_file("dist config", &prefix.path().join(&rel_config_path)));
        let config = try!(Config::parse(config_str));
        try!(tx.remove_file("dist config", rel_config_path));

        for component in config.components {
            tx = try!(self.uninstall_component(&component, tx, notify_handler));
        }
        tx.commit();

        Ok(())
    }

    fn uninstall_component<'a>(&self, component: &Component, mut tx: Transaction<'a>,
                               notify_handler: NotifyHandler) -> Result<Transaction<'a>> {
        // For historical reasons, the rust-installer component
        // names are not the same as the dist manifest component
        // names. Some are just the component name some are the
        // component name plus the target triple.
        let ref name = format!("{}-{}", component.pkg, component.target);
        let ref short_name = format!("{}", component.pkg);
        if let Some(c) = try!(self.installation.find(&name)) {
            tx = try!(c.uninstall(tx));
        } else if let Some(c) = try!(self.installation.find(&short_name)) {
            tx = try!(c.uninstall(tx));
        } else {
            notify_handler.call(Notification::MissingInstalledComponent(&name));
        }

        Ok(tx)
    }

}

/// Returns components to uninstall, install, and the list of all
/// components that will be up to date after the update.
fn build_update_component_lists(
    new_manifest: &Manifest,
    old_manifest: &Option<Manifest>,
    config: &Option<Config>,
    changes: Changes,
    rust_target_package: &TargettedPackage,
    ) -> Result<(Vec<Component>, Vec<Component>, Vec<Component>)> {

    // Check some invariantns
    for component_to_add in &changes.add_extensions {
        assert!(rust_target_package.extensions.contains(component_to_add),
                "package must contain extension to add");
        assert!(!changes.remove_extensions.contains(component_to_add),
                "can't both add and remove extensions");
    }
    for component_to_remove in &changes.remove_extensions {
        assert!(rust_target_package.extensions.contains(component_to_remove),
                "package must contain extension to remove");
        let config = config.as_ref().expect("removing extension on fresh install?");
        assert!(config.components.contains(component_to_remove),
                "removing package that isn't installed");
    }

    // The list of components already installed, empty if a new install
    let starting_list = config.as_ref().map(|c| c.components.clone()).unwrap_or(Vec::new());

    // The list of components we'll have installed at the end
    let mut final_component_list = Vec::new();

    // The lists of components to uninstall and to install
    let mut components_to_uninstall = Vec::new();
    let mut components_to_install = Vec::new();

    // Find the final list of components we want to be left with when
    // we're done: required components, added extensions, and existing
    // installed extensions.

    // Add components required by the package, according to the
    // manifest
    for required_component in &rust_target_package.components {
        final_component_list.push(required_component.clone());
    }

    // Add requested extension components
    for extension in &changes.add_extensions {
        final_component_list.push(extension.clone());
    }

    // Add extensions that are already installed
    for existing_component in &starting_list {
        let is_extension = rust_target_package.extensions.contains(existing_component);
        let is_removed = changes.remove_extensions.contains(existing_component);
        let is_already_included = final_component_list.contains(existing_component);

        if is_extension && !is_removed && !is_already_included{
            final_component_list.push(existing_component.clone());
        }
    }

    // If this is a full upgrade then the list of components to
    // uninstall is all that are currently installed, and those
    // to install the final list. It's a complete reinstall.
    //
    // If it's a modification then the components to uninstall are
    // those that are currently installed but not in the final list.
    // To install are those on the final list but not already
    // installed.
    let just_modifying_existing_install = old_manifest.as_ref() == Some(new_manifest);
    if !just_modifying_existing_install {
        components_to_uninstall = starting_list.clone();
        components_to_install = final_component_list.clone();
    } else {
        for existing_component in &starting_list {
            if !final_component_list.contains(existing_component) {
                components_to_uninstall.push(existing_component.clone())
            }
        }
        for component in &final_component_list {
            if !starting_list.contains(component) {
                components_to_install.push(component.clone());
            }
        }
    }

    Ok((components_to_uninstall, components_to_install, final_component_list))
}
