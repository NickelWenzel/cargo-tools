export enum CargoProfile {
    none = 'none',
    dev = 'dev',
    release = 'release',
    test = 'test',
    bench = 'bench'
}

export namespace CargoProfile {
    export function fromString(str: string): CargoProfile {
        switch (str.toLowerCase()) {
            case 'release':
                return CargoProfile.release;
            case 'test':
                return CargoProfile.test;
            case 'bench':
                return CargoProfile.bench;
            case 'dev':
            case 'debug':
                return CargoProfile.dev;
            case 'none':
            case '':
                return CargoProfile.none;
            default:
                return CargoProfile.dev;
        }
    }

    export function toString(profile: CargoProfile): string {
        return profile.toString();
    }

    export function getDisplayName(profile: CargoProfile): string {
        switch (profile) {
            case CargoProfile.none:
                return 'No selection';
            case CargoProfile.dev:
                return 'Development';
            case CargoProfile.release:
                return 'Release';
            case CargoProfile.test:
                return 'Test';
            case CargoProfile.bench:
                return 'Bench';
            default:
                return 'Development';
        }
    }

    export function getDescription(profile: CargoProfile): string {
        switch (profile) {
            case CargoProfile.none:
                return 'No profile selection - use default cargo behavior';
            case CargoProfile.dev:
                return 'Development profile (--profile dev) with debug information and fast compilation';
            case CargoProfile.release:
                return 'Release profile (--profile release) with optimizations and smaller binary size';
            case CargoProfile.test:
                return 'Test profile (--profile test) optimized for running tests';
            case CargoProfile.bench:
                return 'Bench profile (--profile bench) optimized for running benchmarks';
            default:
                return 'Development profile (--profile dev) with debug information and fast compilation';
        }
    }

    export function getAllProfiles(): CargoProfile[] {
        return [CargoProfile.none, CargoProfile.dev, CargoProfile.release, CargoProfile.test, CargoProfile.bench];
    }
}
