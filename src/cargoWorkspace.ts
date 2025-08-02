import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
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
}

export class CargoWorkspace {
    private _workspaceRoot: string;
    private _manifest: CargoManifest | null = null;
    private _targets: CargoTarget[] = [];
    private _currentProfile: CargoProfile = CargoProfile.dev;
    private _currentTarget: CargoTarget | null = null;
    private _onDidChangeProfile = new vscode.EventEmitter<CargoProfile>();
    private _onDidChangeTarget = new vscode.EventEmitter<CargoTarget | null>();
    private _onDidChangeTargets = new vscode.EventEmitter<CargoTarget[]>();

    readonly onDidChangeProfile = this._onDidChangeProfile.event;
    readonly onDidChangeTarget = this._onDidChangeTarget.event;
    readonly onDidChangeTargets = this._onDidChangeTargets.event;

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

    get isWorkspace(): boolean {
        return this._manifest?.workspace !== undefined;
    }

    get workspaceMembers(): string[] {
        return this._manifest?.workspace?.members || [];
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

        try {
            // Use cargo metadata to get accurate target information
            const { stdout } = await execAsync('cargo metadata --format-version 1 --no-deps', {
                cwd: this._workspaceRoot
            });

            const metadata: CargoMetadata = JSON.parse(stdout);

            // Process each package in the workspace
            for (const pkg of metadata.packages) {
                // For single-package workspaces, process all packages that match the workspace root
                // For multi-package workspaces, only process workspace members
                const isWorkspaceMember = metadata.workspace_members.some(member => member.includes(pkg.name)) ||
                    pkg.manifest_path.startsWith(metadata.workspace_root);

                if (!isWorkspaceMember) {
                    continue;
                }

                // Process targets for this package
                for (const target of pkg.targets) {
                    const cargoTarget = new CargoTarget(
                        target.name,
                        target.kind,
                        target.src_path,
                        target.edition || pkg.edition || '2021',
                        pkg.name
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

        if (fs.existsSync(srcDir)) {
            // Check for main.rs (binary target)
            const mainPath = path.join(srcDir, 'main.rs');
            if (fs.existsSync(mainPath)) {
                this._targets.push(new CargoTarget(packageName, ['bin'], mainPath, '2021', packageName));
            }

            // Check for lib.rs (library target)
            const libPath = path.join(srcDir, 'lib.rs');
            if (fs.existsSync(libPath)) {
                const libName = this._manifest?.lib?.name || packageName;
                this._targets.push(new CargoTarget(libName, ['lib'], libPath, '2021', packageName));
            }

            // Check for bin directory (additional binary targets)
            const binDir = path.join(srcDir, 'bin');
            if (fs.existsSync(binDir)) {
                try {
                    const binFiles = await fs.promises.readdir(binDir);
                    for (const file of binFiles) {
                        if (file.endsWith('.rs')) {
                            const name = path.basename(file, '.rs');
                            this._targets.push(new CargoTarget(name, ['bin'], path.join(binDir, file), '2021', packageName));
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
                        this._targets.push(new CargoTarget(name, ['example'], path.join(examplesDir, file), '2021', packageName));
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
                        this._targets.push(new CargoTarget(name, ['test'], path.join(testsDir, file), '2021', packageName));
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
                        this._targets.push(new CargoTarget(name, ['bench'], path.join(benchesDir, file), '2021', packageName));
                    }
                }
            } catch (error) {
                console.error('Failed to read benches directory:', error);
            }
        }

        this._onDidChangeTargets.fire(this._targets);
    }

    private setDefaultTarget(): void {
        if (this._targets.length > 0) {
            // Prefer binary targets over library targets
            const binTarget = this._targets.find(t => t.kind.includes('bin'));
            this._currentTarget = binTarget || this._targets[0];
            this._onDidChangeTarget.fire(this._currentTarget);
        }
    }

    setProfile(profile: CargoProfile): void {
        if (this._currentProfile !== profile) {
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

    async refresh(): Promise<void> {
        await this.loadManifest();
        await this.discoverTargets();
        this.setDefaultTarget();
    }

    async refreshTargets(): Promise<void> {
        await this.discoverTargets();
        this.setDefaultTarget();
    }

    getCargoArgs(command: string, additionalArgs: string[] = []): string[] {
        const args = [command];

        // Add profile
        if (this._currentProfile === CargoProfile.release) {
            args.push('--release');
        }

        // Add target
        if (this._currentTarget && command !== 'clean') {
            if (this._currentTarget.kind.includes('bin')) {
                args.push('--bin', this._currentTarget.name);
            } else if (this._currentTarget.kind.includes('lib')) {
                args.push('--lib');
            }
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
