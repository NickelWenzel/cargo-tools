export enum TargetActionType {
    Build = 'build',
    Run = 'run',
    Debug = 'debug',
    Test = 'test',
    Bench = 'bench'
}

export class CargoTarget {
    constructor(
        public readonly name: string,
        public readonly kind: string[],
        public readonly srcPath: string,
        public readonly edition: string = '2021',
        public readonly packageName?: string,
        public readonly packagePath?: string
    ) { }

    get displayName(): string {
        const kindStr = this.kind.join(', ');
        return `${this.name} (${kindStr})`;
    }

    get isExecutable(): boolean {
        return Boolean(this.kind && Array.isArray(this.kind) && this.kind.includes('bin'));
    }

    get isLibrary(): boolean {
        if (!this.kind || !Array.isArray(this.kind)) {
            return false;
        }

        // All library crate types that should be treated as library targets
        const libraryKinds = ['lib', 'dylib', 'staticlib', 'cdylib', 'rlib'];
        return this.kind.some(kind => libraryKinds.includes(kind));
    }

    get isTest(): boolean {
        return Boolean(this.kind && Array.isArray(this.kind) && this.kind.includes('test'));
    }

    get isBench(): boolean {
        return Boolean(this.kind && Array.isArray(this.kind) && this.kind.includes('bench'));
    }

    get isExample(): boolean {
        return Boolean(this.kind && Array.isArray(this.kind) && this.kind.includes('example'));
    }

    /**
     * Get the action types that this target supports
     */
    get supportedActionTypes(): TargetActionType[] {
        const actions: TargetActionType[] = [];

        // All targets can be built
        actions.push(TargetActionType.Build);

        // Executables (bin and example) can be run and debugged
        if (this.isExecutable || this.isExample) {
            actions.push(TargetActionType.Run);
            actions.push(TargetActionType.Debug);
        }

        // Test targets can be tested
        if (this.isTest) {
            actions.push(TargetActionType.Test);
        }

        // Bench targets can be benchmarked
        if (this.isBench) {
            actions.push(TargetActionType.Bench);
        }

        return actions;
    }

    /**
     * Check if this target supports a specific action type
     */
    supportsActionType(actionType: TargetActionType): boolean {
        return this.supportedActionTypes.includes(actionType);
    }

    /**
     * Get the primary cargo command for a given action type
     */
    getCargoCommand(actionType: TargetActionType): string {
        switch (actionType) {
            case TargetActionType.Build:
                return 'build';
            case TargetActionType.Run:
                return 'run';
            case TargetActionType.Debug:
                return 'build'; // Debug builds first, then launches debugger
            case TargetActionType.Test:
                return 'test';
            case TargetActionType.Bench:
                return 'bench';
            default:
                throw new Error(`Unsupported action type: ${actionType}`);
        }
    }

    /**
     * Get the target-specific arguments for a given action type
     */
    getTargetArgs(actionType: TargetActionType): string[] {
        const args: string[] = [];

        if (this.isLibrary && actionType === TargetActionType.Build) {
            args.push('--lib');
        } else if (this.isExecutable && (actionType === TargetActionType.Build || actionType === TargetActionType.Run)) {
            args.push('--bin', this.name);
        } else if (this.isExample && (actionType === TargetActionType.Build || actionType === TargetActionType.Run)) {
            args.push('--example', this.name);
        } else if (this.isTest && actionType === TargetActionType.Test) {
            args.push('--test', this.name);
        } else if (this.isBench && actionType === TargetActionType.Bench) {
            args.push('--bench', this.name);
        }

        return args;
    }

    toString(): string {
        return this.displayName;
    }
}
