export class CargoProfile {
    private static readonly STANDARD_PROFILES = ['none', 'dev', 'release', 'test', 'bench', 'doc'];
    private static customProfiles: Set<string> = new Set();

    static readonly none = new CargoProfile('none');
    static readonly dev = new CargoProfile('dev');
    static readonly release = new CargoProfile('release');
    static readonly test = new CargoProfile('test');
    static readonly bench = new CargoProfile('bench');
    static readonly doc = new CargoProfile('doc');

    constructor(private readonly value: string) { }

    toString(): string {
        return this.value;
    }

    static fromString(str: string): CargoProfile {
        const normalized = str.toLowerCase();
        switch (normalized) {
            case 'release':
                return CargoProfile.release;
            case 'test':
                return CargoProfile.test;
            case 'bench':
                return CargoProfile.bench;
            case 'doc':
                return CargoProfile.doc;
            case 'dev':
            case 'debug':
                return CargoProfile.dev;
            case 'none':
            case '':
                return CargoProfile.none;
            default:
                // For custom profiles, create a new instance
                return new CargoProfile(str);
        }
    }

    static addCustomProfile(profileName: string): void {
        if (!this.STANDARD_PROFILES.includes(profileName.toLowerCase())) {
            this.customProfiles.add(profileName);
        }
    }

    static getCustomProfiles(): string[] {
        return Array.from(this.customProfiles).sort();
    }

    static clearCustomProfiles(): void {
        this.customProfiles.clear();
    }

    isCustom(): boolean {
        return !CargoProfile.STANDARD_PROFILES.includes(this.value.toLowerCase());
    }

    equals(other: CargoProfile): boolean {
        return this.value === other.value;
    }
}

export namespace CargoProfile {
    export function toString(profile: CargoProfile): string {
        return profile.toString();
    }

    export function getDisplayName(profile: CargoProfile): string {
        const value = profile.toString();
        switch (value) {
            case 'none':
                return 'No selection';
            case 'dev':
                return 'Development';
            case 'release':
                return 'Release';
            case 'test':
                return 'Test';
            case 'bench':
                return 'Bench';
            case 'doc':
                return 'Doc';
            default:
                // For custom profiles, capitalize first letter
                return value.charAt(0).toUpperCase() + value.slice(1);
        }
    }

    export function getDescription(profile: CargoProfile): string {
        const value = profile.toString();
        switch (value) {
            case 'none':
                return 'No profile selection - use default cargo behavior';
            case 'dev':
                return 'Development profile (--profile dev) with debug information and fast compilation';
            case 'release':
                return 'Release profile (--profile release) with optimizations and smaller binary size';
            case 'test':
                return 'Test profile (--profile test) optimized for running tests';
            case 'bench':
                return 'Bench profile (--profile bench) optimized for running benchmarks';
            case 'doc':
                return 'Doc profile (--profile doc) optimized for documentation generation';
            default:
                return `Custom profile (--profile ${value}) with user-defined settings`;
        }
    }

    export function getAllProfiles(): CargoProfile[] {
        const standardProfiles = [
            CargoProfile.none,
            CargoProfile.dev,
            CargoProfile.release,
            CargoProfile.test,
            CargoProfile.bench
            // Note: CargoProfile.doc is excluded from selection as it's not typically user-selectable
        ];

        const customProfileNames = CargoProfile.getCustomProfiles();
        const customProfiles = customProfileNames.map(name => new CargoProfile(name));

        return [...standardProfiles, ...customProfiles];
    }
}
