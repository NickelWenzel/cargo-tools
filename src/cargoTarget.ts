export enum TargetActionType {
    Build = 'build',
    Run = 'run',
    Debug = 'debug',
    Test = 'test',
    Bench = 'bench'
}

export enum CargoTargetKind {
    Bin = 'bin',
    Lib = 'lib',
    Bench = 'bench',
    Example = 'example',
    Test = 'test',
    Unknown = 'unknown',
}

export function toTargetKind(kinds: string[]): CargoTargetKind {
    if (kinds.includes('bin')) {
        return CargoTargetKind.Bin;
    } else if (kinds.includes('lib') || kinds.includes('rlib') || kinds.includes('dylib') || kinds.includes('cdylib') || kinds.includes('staticlib') || kinds.includes('proc-macro')) {
        return CargoTargetKind.Lib;
    } else if (kinds.includes('example')) {
        return CargoTargetKind.Example;
    } else if (kinds.includes('test')) {
        return CargoTargetKind.Test;
    } else if (kinds.includes('bench')) {
        return CargoTargetKind.Bench;
    } else {
        return CargoTargetKind.Unknown; // Default fallback
    }
}

export class CargoTarget {
    constructor(
        public readonly name: string,
        public readonly kind: CargoTargetKind,
        public readonly srcPath: string,
        public readonly edition: string = '2021',
        public readonly packageName?: string,
        public readonly packagePath?: string
    ) { }

    get displayName(): string {
        return `${this.name} (${this.kind})`;
    }

    /**
     * Create a unique identifier for this target that can be used for persistence
     */
    get id(): string {
        return `${this.kind}:${this.name}:${this.packageName || ''}`;
    }

    /**
     * Create a CargoTarget from a plain object (used for state restoration)
     */
    static fromObject(obj: any): CargoTarget {
        return new CargoTarget(
            obj.name,
            obj.kind,
            obj.srcPath,
            obj.edition || '2021',
            obj.packageName,
            obj.packagePath
        );
    }

    get isExecutable(): boolean {
        return this.kind === CargoTargetKind.Bin;
    }

    get isLibrary(): boolean {
        return this.kind === CargoTargetKind.Lib;
    }

    get isTest(): boolean {
        return this.kind === CargoTargetKind.Test;
    }

    get isBench(): boolean {
        return this.kind === CargoTargetKind.Bench;
    }

    get isExample(): boolean {
        return this.kind === CargoTargetKind.Example;
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
