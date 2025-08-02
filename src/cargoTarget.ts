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
        return this.kind && Array.isArray(this.kind) && this.kind.includes('bin');
    }

    get isLibrary(): boolean {
        return this.kind && Array.isArray(this.kind) && this.kind.includes('lib');
    }

    get isTest(): boolean {
        return this.kind && Array.isArray(this.kind) && this.kind.includes('test');
    }

    get isBench(): boolean {
        return this.kind && Array.isArray(this.kind) && this.kind.includes('bench');
    }

    get isExample(): boolean {
        return this.kind && Array.isArray(this.kind) && this.kind.includes('example');
    }

    toString(): string {
        return this.displayName;
    }
}
