export enum CargoProfile {
    dev = 'dev',
    release = 'release'
}

export namespace CargoProfile {
    export function fromString(str: string): CargoProfile {
        switch (str.toLowerCase()) {
            case 'release':
                return CargoProfile.release;
            case 'dev':
            case 'debug':
            default:
                return CargoProfile.dev;
        }
    }

    export function toString(profile: CargoProfile): string {
        return profile.toString();
    }

    export function getDisplayName(profile: CargoProfile): string {
        switch (profile) {
            case CargoProfile.dev:
                return 'Development';
            case CargoProfile.release:
                return 'Release';
        }
    }

    export function getDescription(profile: CargoProfile): string {
        switch (profile) {
            case CargoProfile.dev:
                return 'Development profile with debug information and fast compilation';
            case CargoProfile.release:
                return 'Release profile with optimizations and smaller binary size';
        }
    }

    export function getAllProfiles(): CargoProfile[] {
        return [CargoProfile.dev, CargoProfile.release];
    }
}
