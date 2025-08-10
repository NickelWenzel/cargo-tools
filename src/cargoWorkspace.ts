import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import * as os from 'os';
import { CargoTarget } from './cargoTarget';
import { CargoProfile } from './cargoProfile';
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

interface CargoMetadataTarget {
    name: string;
    kind: string[];
    crate_types: string[];
    src_path: string;
    edition: string;
    doctest: boolean;
    test: boolean;
}

interface CargoMetadataPackage {
    name: string;
    version: string;
    edition: string;
    manifest_path: string;
    targets: CargoMetadataTarget[];
    features?: Record<string, string[]>;
}

interface CargoMetadata {
    packages: CargoMetadataPackage[];
    workspace_members: string[];
    workspace_root: string;
}

export interface CargoManifest {
    package?: {
        name: string;
        version: string;
        edition?: string;
    };
    workspace?: {
        members: string[];
        'default-members'?: string[];
    };
    bin?: Array<{ name: string; path?: string }>;
    lib?: { name?: string; path?: string };
    dependencies?: Record<string, any>;
    features?: Record<string, string[]>;
    profiles?: Record<string, any>;
}

export class CargoWorkspace {
    private _workspaceRoot: string;
    private _manifest: CargoManifest | null = null;
    private _targets: CargoTarget[] = [];
    private _currentProfile: CargoProfile = CargoProfile.none;
    private _currentTarget: CargoTarget | null = null;
    private _selectedPackage: string | undefined = undefined; // undefined means "No selection"
    private _workspacePackageNames: string[] = []; // Package names from cargo metadata
    private _packageFeatures: Map<string, string[]> = new Map(); // Features available for each package
    private _selectedBuildTarget: string | null = null; // null means "No selection"
    private _selectedRunTarget: string | null = null; // Selected run target
    private _selectedBenchmarkTarget: string | null = null; // Selected benchmark target
    private _selectedPlatformTarget: string | null = null; // Selected platform target (e.g., x86_64-unknown-linux-gnu)
    private _selectedFeatures: Set<string> = new Set(); // Selected features, default to none (no features selected)
    private _onDidChangeProfile = new vscode.EventEmitter<CargoProfile>();
    private _onDidChangeTarget = new vscode.EventEmitter<CargoTarget | null>();
    private _onDidChangeTargets = new vscode.EventEmitter<CargoTarget[]>();
    private _onDidChangeSelectedPackage = new vscode.EventEmitter<string | undefined>();
    private _onDidChangeSelectedBuildTarget = new vscode.EventEmitter<string | null>();
    private _onDidChangeSelectedRunTarget = new vscode.EventEmitter<string | null>();
    private _onDidChangeSelectedBenchmarkTarget = new vscode.EventEmitter<string | null>();
    private _onDidChangeSelectedPlatformTarget = new vscode.EventEmitter<string | null>();
    private _onDidChangeSelectedFeatures = new vscode.EventEmitter<Set<string>>();

    readonly onDidChangeProfile = this._onDidChangeProfile.event;
    readonly onDidChangeTarget = this._onDidChangeTarget.event;
    readonly onDidChangeTargets = this._onDidChangeTargets.event;
    readonly onDidChangeSelectedPackage = this._onDidChangeSelectedPackage.event;
    readonly onDidChangeSelectedBuildTarget = this._onDidChangeSelectedBuildTarget.event;
    readonly onDidChangeSelectedRunTarget = this._onDidChangeSelectedRunTarget.event;
    readonly onDidChangeSelectedBenchmarkTarget = this._onDidChangeSelectedBenchmarkTarget.event;
    readonly onDidChangeSelectedPlatformTarget = this._onDidChangeSelectedPlatformTarget.event;
    readonly onDidChangeSelectedFeatures = this._onDidChangeSelectedFeatures.event;

    constructor(workspaceRoot: string) {
        this._workspaceRoot = workspaceRoot;
    }

    get workspaceRoot(): string {
        return this._workspaceRoot;
    }

    get manifest(): CargoManifest | null {
        return this._manifest;
    }

    get targets(): CargoTarget[] {
        return this._targets;
    }

    get currentProfile(): CargoProfile {
        return this._currentProfile;
    }

    get currentTarget(): CargoTarget | null {
        return this._currentTarget;
    }

    get selectedPackage(): string | undefined {
        return this._selectedPackage;
    }

    get selectedBuildTarget(): string | null {
        return this._selectedBuildTarget;
    }

    get selectedRunTarget(): string | null {
        return this._selectedRunTarget;
    }

    get selectedBenchmarkTarget(): string | null {
        return this._selectedBenchmarkTarget;
    }

    get selectedPlatformTarget(): string | null {
        return this._selectedPlatformTarget;
    }

    get selectedFeatures(): Set<string> {
        return this._selectedFeatures;
    }

    get isWorkspace(): boolean {
        return this._manifest?.workspace !== undefined;
    }

    get projectName(): string {
        // For workspace projects, use the workspace root directory name
        if (this.isWorkspace) {
            return path.basename(this._workspaceRoot);
        }
        // For single-package projects, use the package name from manifest
        return this._manifest?.package?.name || path.basename(this._workspaceRoot);
    }

    get workspaceMembers(): string[] {
        // Use package names from cargo metadata if available, fallback to TOML parsing
        return this._workspacePackageNames.length > 0
            ? this._workspacePackageNames
            : this._manifest?.workspace?.members || [];
    }

    getWorkspaceMembers(): Map<string, CargoTarget[]> {
        const members = new Map<string, CargoTarget[]>();

        for (const target of this._targets) {
            const memberName = target.packageName || 'default';

            if (!members.has(memberName)) {
                members.set(memberName, []);
            }
            members.get(memberName)!.push(target);
        }

        return members;
    }

    async initialize(): Promise<void> {
        await this.loadManifest();
        await this.discoverTargets();
        await this.discoverCustomProfiles();
        this.setDefaultTarget();
    }

    private async loadManifest(): Promise<void> {
        const manifestPath = path.join(this._workspaceRoot, 'Cargo.toml');

        try {
            const content = await fs.promises.readFile(manifestPath, 'utf8');
            // Simple TOML parsing - for production, consider using a proper TOML parser
            this._manifest = this.parseToml(content);
        } catch (error) {
            console.error('Failed to load Cargo.toml:', error);
            this._manifest = null;
        }
    }

    private parseToml(content: string): CargoManifest {
        // Basic TOML parsing - this is simplified and would need a proper TOML parser for production
        const lines = content.split('\n');
        const manifest: CargoManifest = {};
        let currentSection: string | null = null;
        let currentObject: any = manifest;

        for (const line of lines) {
            const trimmed = line.trim();
            if (!trimmed || trimmed.startsWith('#')) {
                continue;
            }

            // Section headers
            const sectionMatch = trimmed.match(/^\[([^\]]+)\]$/);
            if (sectionMatch) {
                currentSection = sectionMatch[1];
                const parts = currentSection.split('.');
                currentObject = manifest;

                for (const part of parts) {
                    if (!currentObject[part]) {
                        currentObject[part] = {};
                    }
                    currentObject = currentObject[part];
                }
                continue;
            }

            // Key-value pairs
            const keyValueMatch = trimmed.match(/^([^=]+)=(.+)$/);
            if (keyValueMatch && currentObject) {
                const key = keyValueMatch[1].trim();
                let value = keyValueMatch[2].trim();

                // Remove quotes
                if ((value.startsWith('"') && value.endsWith('"')) ||
                    (value.startsWith("'") && value.endsWith("'"))) {
                    value = value.slice(1, -1);
                }

                currentObject[key] = value;
            }
        }

        return manifest;
    }

    private async discoverTargets(): Promise<void> {
        this._targets = [];
        this._workspacePackageNames = [];
        this._packageFeatures.clear(); // Clear package features before discovery

        try {
            // Use cargo metadata to get accurate target information
            const { stdout } = await execAsync('cargo metadata --format-version 1 --no-deps', {
                cwd: this._workspaceRoot
            });

            const metadata: CargoMetadata = JSON.parse(stdout);

            // Extract workspace package names from metadata
            const workspacePackageNames = new Set<string>();
            for (const pkg of metadata.packages) {
                const isWorkspaceMember = metadata.workspace_members.some(member => member.includes(pkg.name)) ||
                    pkg.manifest_path.startsWith(metadata.workspace_root);
                if (isWorkspaceMember) {
                    workspacePackageNames.add(pkg.name);
                }
            }
            this._workspacePackageNames = Array.from(workspacePackageNames).sort();

            // Process each package in the workspace
            for (const pkg of metadata.packages) {
                // For single-package workspaces, process all packages that match the workspace root
                // For multi-package workspaces, only process workspace members
                const isWorkspaceMember = metadata.workspace_members.some(member => member.includes(pkg.name)) ||
                    pkg.manifest_path.startsWith(metadata.workspace_root);

                if (!isWorkspaceMember) {
                    continue;
                }

                // Collect features for this package
                if (pkg.features) {
                    this._packageFeatures.set(pkg.name, Object.keys(pkg.features));
                }

                // Process targets for this package
                for (const target of pkg.targets) {
                    // Get package directory from manifest path
                    const packagePath = path.dirname(pkg.manifest_path);

                    const cargoTarget = new CargoTarget(
                        target.name,
                        Array.isArray(target.kind) ? target.kind : [target.kind || 'bin'],
                        target.src_path,
                        target.edition || pkg.edition || '2021',
                        pkg.name,
                        packagePath
                    );
                    this._targets.push(cargoTarget);
                }
            }

            this._onDidChangeTargets.fire(this._targets);
        } catch (error) {
            console.error('Failed to discover targets using cargo metadata:', error);
            // Fallback to manual discovery
            await this.discoverTargetsManually();
        }
    }

    private async discoverTargetsManually(): Promise<void> {
        // Fallback manual discovery when cargo metadata fails
        const srcDir = path.join(this._workspaceRoot, 'src');
        const packageName = this._manifest?.package?.name || path.basename(this._workspaceRoot);
        const packagePath = this._workspaceRoot; // For manual discovery, package path is workspace root

        if (fs.existsSync(srcDir)) {
            // Check for main.rs (binary target)
            const mainPath = path.join(srcDir, 'main.rs');
            if (fs.existsSync(mainPath)) {
                this._targets.push(new CargoTarget(packageName, ['bin'], mainPath, '2021', packageName, packagePath));
            }

            // Check for lib.rs (library target)
            const libPath = path.join(srcDir, 'lib.rs');
            if (fs.existsSync(libPath)) {
                const libName = this._manifest?.lib?.name || packageName;
                this._targets.push(new CargoTarget(libName, ['lib'], libPath, '2021', packageName, packagePath));
            }

            // Check for bin directory (additional binary targets)
            const binDir = path.join(srcDir, 'bin');
            if (fs.existsSync(binDir)) {
                try {
                    const binFiles = await fs.promises.readdir(binDir);
                    for (const file of binFiles) {
                        if (file.endsWith('.rs')) {
                            const name = path.basename(file, '.rs');
                            this._targets.push(new CargoTarget(name, ['bin'], path.join(binDir, file), '2021', packageName, packagePath));
                        }
                    }
                } catch (error) {
                    console.error('Failed to read bin directory:', error);
                }
            }
        }

        // Check for examples directory
        const examplesDir = path.join(this._workspaceRoot, 'examples');
        if (fs.existsSync(examplesDir)) {
            try {
                const exampleFiles = await fs.promises.readdir(examplesDir);
                for (const file of exampleFiles) {
                    if (file.endsWith('.rs')) {
                        const name = path.basename(file, '.rs');
                        this._targets.push(new CargoTarget(name, ['example'], path.join(examplesDir, file), '2021', packageName, packagePath));
                    }
                }
            } catch (error) {
                console.error('Failed to read examples directory:', error);
            }
        }

        // Check for tests directory
        const testsDir = path.join(this._workspaceRoot, 'tests');
        if (fs.existsSync(testsDir)) {
            try {
                const testFiles = await fs.promises.readdir(testsDir);
                for (const file of testFiles) {
                    if (file.endsWith('.rs')) {
                        const name = path.basename(file, '.rs');
                        this._targets.push(new CargoTarget(name, ['test'], path.join(testsDir, file), '2021', packageName, packagePath));
                    }
                }
            } catch (error) {
                console.error('Failed to read tests directory:', error);
            }
        }

        // Check for benches directory
        const benchesDir = path.join(this._workspaceRoot, 'benches');
        if (fs.existsSync(benchesDir)) {
            try {
                const benchFiles = await fs.promises.readdir(benchesDir);
                for (const file of benchFiles) {
                    if (file.endsWith('.rs')) {
                        const name = path.basename(file, '.rs');
                        this._targets.push(new CargoTarget(name, ['bench'], path.join(benchesDir, file), '2021', packageName, packagePath));
                    }
                }
            } catch (error) {
                console.error('Failed to read benches directory:', error);
            }
        }

        this._onDidChangeTargets.fire(this._targets);
    }

    private async discoverCustomProfiles(): Promise<void> {
        // Clear existing custom profiles
        CargoProfile.clearCustomProfiles();

        try {
            // 1. Parse workspace/root level Cargo.toml for profiles
            await this.parseProfilesFromCargoToml();

            // 2. Parse .cargo/config.toml in workspace if it exists
            await this.parseProfilesFromConfigToml(path.join(this._workspaceRoot, '.cargo', 'config.toml'));

            // 3. Parse $CARGO_HOME/config.toml if it exists
            const cargoHome = process.env.CARGO_HOME || path.join(os.homedir(), '.cargo');
            await this.parseProfilesFromConfigToml(path.join(cargoHome, 'config.toml'));

        } catch (error) {
            console.error('Failed to discover custom profiles:', error);
        }
    }

    private async parseProfilesFromCargoToml(): Promise<void> {
        // Check both possible locations for profiles
        const profilesFromProfilesSection = this._manifest?.profiles;
        const profilesFromProfileSection = (this._manifest as any)?.profile;

        if (profilesFromProfilesSection) {
            for (const profileName of Object.keys(profilesFromProfilesSection)) {
                CargoProfile.addCustomProfile(profileName);
            }
        }

        if (profilesFromProfileSection) {
            for (const profileName of Object.keys(profilesFromProfileSection)) {
                CargoProfile.addCustomProfile(profileName);
            }
        }
    }

    private async parseProfilesFromConfigToml(configPath: string): Promise<void> {
        try {
            if (fs.existsSync(configPath)) {
                const content = await fs.promises.readFile(configPath, 'utf8');
                const config = this.parseToml(content) as any;

                // Config.toml uses "profile" (singular) while Cargo.toml uses "profiles" (plural)
                const profilesSection = config.profile || config.profiles;
                if (profilesSection) {
                    for (const profileName of Object.keys(profilesSection)) {
                        CargoProfile.addCustomProfile(profileName);
                    }
                }
            }
        } catch (error) {
            console.error(`Failed to parse config.toml at ${configPath}:`, error);
        }
    }

    private setDefaultTarget(): void {
        if (this._targets.length > 0) {
            // Prefer binary targets over library targets
            const binTarget = this._targets.find(t => t.kind && Array.isArray(t.kind) && t.kind.includes('bin'));
            this._currentTarget = binTarget || this._targets[0];
            this._onDidChangeTarget.fire(this._currentTarget);
        }
    }

    setProfile(profile: CargoProfile): void {
        if (!this._currentProfile.equals(profile)) {
            this._currentProfile = profile;
            this._onDidChangeProfile.fire(this._currentProfile);
        }
    }

    setTarget(target: CargoTarget | null): void {
        if (this._currentTarget !== target) {
            this._currentTarget = target;
            this._onDidChangeTarget.fire(this._currentTarget);
        }
    }

    setSelectedPackage(packageName: string | undefined): void {
        if (this._selectedPackage !== packageName) {
            this._selectedPackage = packageName;

            // Reset target selections when package changes
            // because targets are package-specific
            this.resetTargetSelections();

            this._onDidChangeSelectedPackage.fire(this._selectedPackage);
        }
    }

    setSelectedBuildTarget(targetName: string | null): void {
        if (this._selectedBuildTarget !== targetName) {
            this._selectedBuildTarget = targetName;
            this._onDidChangeSelectedBuildTarget.fire(this._selectedBuildTarget);
        }
    }

    setSelectedRunTarget(targetName: string | null): void {
        if (this._selectedRunTarget !== targetName) {
            this._selectedRunTarget = targetName;
            this._onDidChangeSelectedRunTarget.fire(this._selectedRunTarget);
        }
    }

    setSelectedBenchmarkTarget(targetName: string | null): void {
        if (this._selectedBenchmarkTarget !== targetName) {
            this._selectedBenchmarkTarget = targetName;
            this._onDidChangeSelectedBenchmarkTarget.fire(this._selectedBenchmarkTarget);
        }
    }

    setSelectedPlatformTarget(targetTriple: string | null): void {
        if (this._selectedPlatformTarget !== targetTriple) {
            this._selectedPlatformTarget = targetTriple;
            this._onDidChangeSelectedPlatformTarget.fire(this._selectedPlatformTarget);
        }
    }

    private resetTargetSelections(): void {
        // Reset all target selections when package changes
        // This ensures that selected targets are valid for the new package context

        const oldBuildTarget = this._selectedBuildTarget;
        const oldRunTarget = this._selectedRunTarget;
        const oldBenchmarkTarget = this._selectedBenchmarkTarget;
        const oldFeatures = new Set(this._selectedFeatures);

        this._selectedBuildTarget = null;
        this._selectedRunTarget = null;
        this._selectedBenchmarkTarget = null;
        this._selectedFeatures = new Set(); // Reset to default (no features selected)

        // Fire events only if there were actual changes
        if (oldBuildTarget !== null) {
            this._onDidChangeSelectedBuildTarget.fire(this._selectedBuildTarget);
        }
        if (oldRunTarget !== null) {
            this._onDidChangeSelectedRunTarget.fire(this._selectedRunTarget);
        }
        if (oldBenchmarkTarget !== null) {
            this._onDidChangeSelectedBenchmarkTarget.fire(this._selectedBenchmarkTarget);
        }

        // Check if features actually changed
        if (oldFeatures.size !== this._selectedFeatures.size ||
            !Array.from(oldFeatures).every(f => this._selectedFeatures.has(f))) {
            this._onDidChangeSelectedFeatures.fire(this._selectedFeatures);
        }
    }

    /**
     * Get available features for a specific package
     */
    getPackageFeatures(packageName: string): string[] {
        return this._packageFeatures.get(packageName) || [];
    }

    /**
     * Get all available features for the current package context
     */
    getAvailableFeatures(): string[] {
        const features = ['all-features']; // Always include all-features option

        if (this._selectedPackage) {
            // When a specific package is selected, show its features
            const packageFeatures = this.getPackageFeatures(this._selectedPackage);
            features.push(...packageFeatures);
        }
        // When no selection, only show "all-features"

        return features;
    }

    /**
     * Set selected features
     */
    setSelectedFeatures(features: Set<string>): void {
        this._selectedFeatures = new Set(features);
        this._onDidChangeSelectedFeatures.fire(this._selectedFeatures);
    }

    /**
     * Toggle a feature selection
     */
    toggleFeature(feature: string): void {
        const newFeatures = new Set(this._selectedFeatures);

        if (feature === 'all-features') {
            // If toggling all-features
            if (newFeatures.has('all-features')) {
                // If all-features is currently selected, deselect it (allow empty selection)
                newFeatures.clear();
            } else {
                // If all-features is not selected, select it and clear others
                newFeatures.clear();
                newFeatures.add('all-features');
            }
        } else {
            // If toggling a specific feature
            if (newFeatures.has(feature)) {
                // Deselect the feature
                newFeatures.delete(feature);
            } else {
                // Select the feature and remove all-features
                newFeatures.delete('all-features');
                newFeatures.add(feature);
            }
            // Note: Empty selection is now allowed as the default state
        }

        this.setSelectedFeatures(newFeatures);
    }

    /**
     * Get installed platform targets from rustup
     */
    async getInstalledPlatformTargets(): Promise<string[]> {
        try {
            const { stdout } = await execAsync('rustup target list --installed', {
                cwd: this._workspaceRoot
            });

            return stdout
                .trim()
                .split('\n')
                .map(line => line.trim())
                .filter(line => line.length > 0);
        } catch (error) {
            console.error('Failed to get installed platform targets:', error);
            return [];
        }
    }

    /**
     * Get all available platform targets from rustup (not just installed)
     */
    async getAvailablePlatformTargets(): Promise<string[]> {
        try {
            const { stdout } = await execAsync('rustup target list', {
                cwd: this._workspaceRoot
            });

            return stdout
                .trim()
                .split('\n')
                .map(line => {
                    // Remove "(installed)" suffix if present
                    return line.replace(/\s*\(installed\)\s*$/, '').trim();
                })
                .filter(line => line.length > 0);
        } catch (error) {
            console.error('Failed to get available platform targets:', error);
            return [];
        }
    }

    /**
     * Get platform targets that are available but not yet installed
     */
    async getAvailableUninstalledPlatformTargets(): Promise<string[]> {
        try {
            const { stdout } = await execAsync('rustup target list', {
                cwd: this._workspaceRoot
            });

            return stdout
                .trim()
                .split('\n')
                .map(line => line.trim())
                .filter(line => line.length > 0 && !line.includes('(installed)'));
        } catch (error) {
            console.error('Failed to get uninstalled platform targets:', error);
            return [];
        }
    }

    /**
     * Install a platform target using rustup
     */
    async installPlatformTarget(targetTriple: string): Promise<boolean> {
        try {
            await execAsync(`rustup target add ${targetTriple}`, {
                cwd: this._workspaceRoot
            });
            return true;
        } catch (error) {
            console.error(`Failed to install platform target ${targetTriple}:`, error);
            return false;
        }
    }

    /**
     * Get the default host platform target
     */
    async getDefaultPlatformTarget(): Promise<string> {
        try {
            const { stdout } = await execAsync('rustc -vV', {
                cwd: this._workspaceRoot
            });

            const hostMatch = stdout.match(/host: (.+)/);
            return hostMatch ? hostMatch[1].trim() : '';
        } catch (error) {
            console.error('Failed to get default platform target:', error);
            return '';
        }
    }

    async refresh(): Promise<void> {
        await this.loadManifest();
        await this.discoverTargets();
        await this.discoverCustomProfiles();
        this.setDefaultTarget();
    }

    async refreshTargets(): Promise<void> {
        await this.discoverTargets();
        await this.discoverCustomProfiles();
        this.setDefaultTarget();
    }

    getCargoArgs(command: string, additionalArgs: string[] = []): string[] {
        const args = [command];

        // Add profile - use --profile flag for all profiles except "none"
        if (!this._currentProfile.equals(CargoProfile.none)) {
            args.push('--profile', this._currentProfile.toString());
        }

        // Add package argument if a specific package is selected and we're in a workspace
        if (this._selectedPackage && this.isWorkspace) {
            args.push('--package', this._selectedPackage);
        }

        // Add target
        if (this._currentTarget && command !== 'clean') {
            if (this._currentTarget.kind && Array.isArray(this._currentTarget.kind)) {
                if (this._currentTarget.kind.includes('bin')) {
                    args.push('--bin', this._currentTarget.name);
                } else if (this._currentTarget.kind.includes('lib')) {
                    args.push('--lib');
                }
            }
        }

        // Add platform target if selected
        if (this._selectedPlatformTarget) {
            args.push('--target', this._selectedPlatformTarget);
        }

        // Add configuration-based arguments
        const config = vscode.workspace.getConfiguration('cargoTools');

        const features = config.get<string[]>('features', []);
        if (features.length > 0) {
            args.push('--features', features.join(','));
        }

        if (config.get<boolean>('allFeatures', false)) {
            args.push('--all-features');
        }

        if (config.get<boolean>('noDefaultFeatures', false)) {
            args.push('--no-default-features');
        }

        // Add command-specific arguments
        const commandArgs = config.get<string[]>(`${command}Args`, []);
        args.push(...commandArgs);

        // Add additional arguments
        args.push(...additionalArgs);

        return args;
    }

    async executeCargoCommand(command: string, additionalArgs: string[] = []): Promise<{ stdout: string; stderr: string }> {
        const cargoPath = vscode.workspace.getConfiguration('cargoTools').get<string>('cargoPath', 'cargo');
        const args = this.getCargoArgs(command, additionalArgs);
        const env = { ...process.env, ...vscode.workspace.getConfiguration('cargoTools').get<Record<string, string>>('environment', {}) };

        try {
            const { stdout, stderr } = await execAsync(`${cargoPath} ${args.join(' ')}`, {
                cwd: this._workspaceRoot,
                env
            });

            return { stdout, stderr };
        } catch (error: any) {
            throw new Error(`Cargo command failed: ${error.message}`);
        }
    }
}
