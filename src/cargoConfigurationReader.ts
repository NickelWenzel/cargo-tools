import * as vscode from 'vscode';

/**
 * Configuration settings interface matching VS Code settings schema
 */
export interface CargoConfiguration {
    cargoPath: string;
    defaultProfile: string;
    buildArgs: string[];
    runArgs: string[];
    testArgs: string[];
    environment: { [key: string]: string };
    features: string[];
    allFeatures: boolean;
    noDefaultFeatures: boolean;
    offline: boolean;
    manifestPath: string | null;
    targetDir: string | null;
    clearOutputBeforeBuild: boolean;
    saveBeforeRun: boolean;
    showOutputOnError: boolean;
    enableLogging: boolean;
    logLevel: 'trace' | 'debug' | 'info' | 'warn' | 'error';
    excludeFolders: string[];
    autoSelectActiveProject: boolean;
    defaultActiveProject: string | null;
}

type EmittersOf<T> = {
    readonly [Key in keyof T]: vscode.EventEmitter<T[Key]>;
};

/**
 * Configuration reader for cargo-tools extension.
 * This class manages VS Code configuration settings and provides reactive updates
 * when settings change, following the pattern from microsoft/vscode-cmake-tools.
 */
export class CargoConfigurationReader implements vscode.Disposable {
    private updateSubscription?: vscode.Disposable;

    constructor(private readonly configData: CargoConfiguration) { }

    dispose() {
        if (this.updateSubscription) {
            this.updateSubscription.dispose();
        }
        // Dispose all event emitters
        for (const emitter of Object.values(this.emitters)) {
            emitter.dispose();
        }
    }

    /**
     * Get a configuration object relevant to the given workspace directory.
     * Supports multiple workspaces having differing configs.
     */
    static create(folder?: vscode.WorkspaceFolder): CargoConfigurationReader {
        const configData = CargoConfigurationReader.loadConfig(folder);
        const reader = new CargoConfigurationReader(configData);

        reader.updateSubscription = vscode.workspace.onDidChangeConfiguration(e => {
            if (e.affectsConfiguration('cargoTools', folder?.uri)) {
                const newConfigData = CargoConfigurationReader.loadConfig(folder);
                reader.update(newConfigData);
            }
        });

        return reader;
    }

    /**
     * Load configuration from VS Code settings
     */
    static loadConfig(folder?: vscode.WorkspaceFolder): CargoConfiguration {
        const config = vscode.workspace.getConfiguration('cargoTools', folder?.uri);

        return {
            cargoPath: config.get<string>('cargoPath', 'cargo'),
            defaultProfile: config.get<string>('defaultProfile', 'dev'),
            buildArgs: config.get<string[]>('buildArgs', []),
            runArgs: config.get<string[]>('runArgs', []),
            testArgs: config.get<string[]>('testArgs', []),
            environment: config.get<{ [key: string]: string }>('environment', {}),
            features: config.get<string[]>('features', []),
            allFeatures: config.get<boolean>('allFeatures', false),
            noDefaultFeatures: config.get<boolean>('noDefaultFeatures', false),
            offline: config.get<boolean>('offline', false),
            manifestPath: config.get<string | null>('manifestPath', null),
            targetDir: config.get<string | null>('targetDir', null),
            clearOutputBeforeBuild: config.get<boolean>('clearOutputBeforeBuild', true),
            saveBeforeRun: config.get<boolean>('saveBeforeRun', true),
            showOutputOnError: config.get<boolean>('showOutputOnError', true),
            enableLogging: config.get<boolean>('enableLogging', false),
            logLevel: config.get<'trace' | 'debug' | 'info' | 'warn' | 'error'>('logLevel', 'info'),
            excludeFolders: config.get<string[]>('excludeFolders', []),
            autoSelectActiveProject: config.get<boolean>('autoSelectActiveProject', true),
            defaultActiveProject: config.get<string | null>('defaultActiveProject', null)
        };
    }

    /**
     * Update configuration with new data and fire events for changed values
     */
    update(newConfigData: CargoConfiguration): string[] {
        return this.updatePartial(newConfigData);
    }

    /**
     * Update configuration partially and return list of changed keys
     */
    updatePartial(newConfigData: Partial<CargoConfiguration>, fireEvent: boolean = true): string[] {
        const keys: string[] = [];
        const oldValues = { ...this.configData };
        Object.assign(this.configData, newConfigData);

        for (const keyObject of Object.getOwnPropertyNames(newConfigData)) {
            const key = keyObject as keyof CargoConfiguration;
            if (!(key in this.emitters)) {
                continue;
            }

            const newValue = this.configData[key];
            const oldValue = oldValues[key];

            // Compare values (simple comparison for now)
            if (JSON.stringify(newValue) !== JSON.stringify(oldValue)) {
                if (fireEvent) {
                    const emitter: vscode.EventEmitter<CargoConfiguration[typeof key]> = this.emitters[key];
                    const temp = newConfigData[key];
                    if (temp !== undefined) {
                        emitter.fire(temp);
                    }
                }
                keys.push(key);
            }
        }

        return keys;
    }

    // Configuration property accessors
    get cargoPath(): string {
        return this.configData.cargoPath;
    }

    get defaultProfile(): string {
        return this.configData.defaultProfile;
    }

    get buildArgs(): string[] {
        return this.configData.buildArgs;
    }

    get runArgs(): string[] {
        return this.configData.runArgs;
    }

    get testArgs(): string[] {
        return this.configData.testArgs;
    }

    get environment(): { [key: string]: string } {
        return this.configData.environment;
    }

    get features(): string[] {
        return this.configData.features;
    }

    get allFeatures(): boolean {
        return this.configData.allFeatures;
    }

    get noDefaultFeatures(): boolean {
        return this.configData.noDefaultFeatures;
    }

    get offline(): boolean {
        return this.configData.offline;
    }

    get manifestPath(): string | null {
        return this.configData.manifestPath;
    }

    get targetDir(): string | null {
        return this.configData.targetDir;
    }

    get clearOutputBeforeBuild(): boolean {
        return this.configData.clearOutputBeforeBuild;
    }

    get saveBeforeRun(): boolean {
        return this.configData.saveBeforeRun;
    }

    get showOutputOnError(): boolean {
        return this.configData.showOutputOnError;
    }

    get enableLogging(): boolean {
        return this.configData.enableLogging;
    }

    get logLevel(): 'trace' | 'debug' | 'info' | 'warn' | 'error' {
        return this.configData.logLevel;
    }

    get excludeFolders(): string[] {
        return this.configData.excludeFolders;
    }

    get autoSelectActiveProject(): boolean {
        return this.configData.autoSelectActiveProject;
    }

    get defaultActiveProject(): string | null {
        return this.configData.defaultActiveProject;
    }

    // Event emitters for configuration changes
    private readonly emitters: EmittersOf<CargoConfiguration> = {
        cargoPath: new vscode.EventEmitter<string>(),
        defaultProfile: new vscode.EventEmitter<string>(),
        buildArgs: new vscode.EventEmitter<string[]>(),
        runArgs: new vscode.EventEmitter<string[]>(),
        testArgs: new vscode.EventEmitter<string[]>(),
        environment: new vscode.EventEmitter<{ [key: string]: string }>(),
        features: new vscode.EventEmitter<string[]>(),
        allFeatures: new vscode.EventEmitter<boolean>(),
        noDefaultFeatures: new vscode.EventEmitter<boolean>(),
        offline: new vscode.EventEmitter<boolean>(),
        manifestPath: new vscode.EventEmitter<string | null>(),
        targetDir: new vscode.EventEmitter<string | null>(),
        clearOutputBeforeBuild: new vscode.EventEmitter<boolean>(),
        saveBeforeRun: new vscode.EventEmitter<boolean>(),
        showOutputOnError: new vscode.EventEmitter<boolean>(),
        enableLogging: new vscode.EventEmitter<boolean>(),
        logLevel: new vscode.EventEmitter<'trace' | 'debug' | 'info' | 'warn' | 'error'>(),
        excludeFolders: new vscode.EventEmitter<string[]>(),
        autoSelectActiveProject: new vscode.EventEmitter<boolean>(),
        defaultActiveProject: new vscode.EventEmitter<string | null>()
    };

    /**
     * Watch for changes on a particular setting
     * @param setting The name of the setting to watch
     * @param cb A callback when the setting changes
     */
    onChange<K extends keyof CargoConfiguration>(setting: K, cb: (value: CargoConfiguration[K]) => any): vscode.Disposable {
        const emitter: vscode.EventEmitter<any> = this.emitters[setting];
        return emitter.event(cb);
    }

    /**
     * Check if a setting has its default value
     */
    isDefaultValue<K extends keyof CargoConfiguration>(setting: K, configurationScope?: vscode.ConfigurationScope): boolean {
        const settings = vscode.workspace.getConfiguration('cargoTools', configurationScope);
        const value = settings.inspect(setting);
        return value?.globalValue === undefined &&
            value?.workspaceValue === undefined &&
            value?.workspaceFolderValue === undefined;
    }
}
