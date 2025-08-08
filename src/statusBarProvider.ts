import * as vscode from 'vscode';
import { CargoConfigurationReader } from './cargoConfigurationReader';
import { CargoProfile } from './cargoProfile';

/**
 * Status bar button visibility settings
 */
export type StatusBarVisibility = 'visible' | 'compact' | 'icon' | 'hidden';

/**
 * Base class for status bar buttons following CMake Tools pattern
 */
abstract class StatusBarButton {
    readonly settingsName: string | null = null;
    protected readonly button: vscode.StatusBarItem;
    private _forceHidden: boolean = false;
    private _hidden: boolean = false;
    private _text: string = '';
    private _tooltip: string | null = null;
    private _icon: string | null = null;

    constructor(protected readonly config: CargoConfigurationReader, protected readonly priority: number) {
        this.button = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, this.priority);
    }

    /**
     * Only used in StatusBar class
     */
    set forceHidden(v: boolean) {
        this._forceHidden = v;
        this.update();
    }

    get hidden() {
        return this._hidden;
    }
    set hidden(v: boolean) {
        this._hidden = v;
        this.update();
    }

    get text(): string {
        return this._text;
    }
    set text(v: string) {
        this._text = v;
        this.update();
    }

    get bracketText(): string {
        return `[${this._text}]`;
    }

    get tooltip(): string | null {
        return this._tooltip;
    }
    set tooltip(v: string | null) {
        this._tooltip = v;
        this.update();
    }

    protected set icon(v: string | null) {
        this._icon = v ? `$(${v})` : null;
    }

    protected set command(v: string | null) {
        this.button.command = v || undefined;
    }

    dispose(): void {
        this.button.dispose();
    }

    update(): void {
        if (!this._isVisible() || this._forceHidden) {
            this.button.hide();
            return;
        }
        const text = this._getText(true);
        if (text === '') {
            this.button.hide();
            return;
        }
        this.button.text = text;
        this.button.tooltip = this._getTooltip() || undefined;
        this.button.show();
    }

    protected getTextNormal(): string {
        if (this._text.length > 0) {
            return this.bracketText;
        }
        return '';
    }
    protected getTextShort(): string {
        return this.getTextNormal();
    }
    protected getTextIcon(): string {
        return '';
    }

    protected getTooltipNormal(): string | null {
        return this._tooltip;
    }

    protected getTooltipShort(): string | null {
        const tooltip = this.getTooltipNormal();
        const text = this.getTextNormal();
        if (!tooltip && !text) {
            return null;
        }
        if (!tooltip || !text) {
            return this.prependCargo(`${tooltip || text}`);
        }
        return this.prependCargo(`${text}\n${tooltip}`);
    }

    protected getTooltipIcon(): string | null {
        return this.getTooltipShort();
    }

    protected isVisible(): boolean {
        return !this.hidden;
    }

    protected prependCargo(text: string | null): any {
        if (!!text) {
            return `Cargo: ${text}`;
        }
        return text;
    }

    private _isVisible(): boolean {
        return this.isVisible() && this._getVisibilitySetting() !== 'hidden';
    }

    private _getVisibilitySetting(): StatusBarVisibility | null {
        if (this.settingsName) {
            // For now, use the general status bar visibility settings
            // In the future, we can add per-button visibility settings
            return 'visible';
        }
        return 'visible';
    }

    private _getTooltip(): string | null {
        const visibility = this._getVisibilitySetting();
        switch (visibility) {
            case 'hidden':
                return null;
            case 'icon':
                return this.getTooltipIcon();
            case 'compact':
                return this.getTooltipShort();
            default:
                return this.getTooltipNormal();
        }
    }

    private _getText(icon: boolean = false): string {
        const type = this._getVisibilitySetting();
        let text: string;
        switch (type) {
            case 'icon':
                text = this.getTextIcon();
                break;
            case 'compact':
                text = this.getTextShort();
                break;
            default:
                text = this.getTextNormal();
                break;
        }
        if (!icon) {
            return text;
        }
        if (!this._icon) {
            return text;
        }
        if (text === '') {
            return this._icon || '';
        }
        return `${this._icon} ${text}`;
    }
}

/**
 * Build Profile Selection Button
 */
class ProfileSelectionButton extends StatusBarButton {
    private static readonly _noProfileSelected = 'No Profile Selected';

    settingsName = 'profile';
    constructor(protected readonly config: CargoConfigurationReader, protected readonly priority: number) {
        super(config, priority);
        this.hidden = false;
        this.command = 'cargo-tools.selectProfile';
        this.icon = 'gear';
        this.tooltip = 'Click to change the active build profile';
    }

    protected getTextNormal(): string {
        const text = this.text;
        if (text.length === 0) {
            return ProfileSelectionButton._noProfileSelected;
        }
        return this.bracketText;
    }

    protected getTextShort(): string {
        const text = this.getTextNormal();
        if (text.length > 20) {
            return `${text.substr(0, 17)}...]`;
        }
        return text;
    }

    protected getTooltipShort(): string | null {
        if (this.getTextNormal() === this.getTextShort()) {
            return this.prependCargo(this.getTooltipNormal());
        }
        return super.getTooltipShort();
    }
}

/**
 * Package Selection Button
 */
class PackageSelectionButton extends StatusBarButton {
    private static readonly _allPackagesSelected = 'All';
    private static readonly _noPackageSelected = 'No Package Selected';

    settingsName = 'package';
    constructor(protected readonly config: CargoConfigurationReader, protected readonly priority: number) {
        super(config, priority);
        this.hidden = false;
        this.command = 'cargo-tools.selectPackage';
        this.icon = 'package';
        this.tooltip = 'Click to change the active package';
    }

    protected getTextNormal(): string {
        const text = this.text;
        if (text.length === 0) {
            return PackageSelectionButton._noPackageSelected;
        }
        if (text === 'All' || text === undefined) {
            return PackageSelectionButton._allPackagesSelected;
        }
        return this.bracketText;
    }

    protected getTextShort(): string {
        const text = this.getTextNormal();
        if (text.length > 20) {
            return `${text.substr(0, 17)}...]`;
        }
        return text;
    }

    protected getTooltipShort(): string | null {
        if (this.getTextNormal() === this.getTextShort()) {
            return this.prependCargo(this.getTooltipNormal());
        }
        return super.getTooltipShort();
    }
}

/**
 * Build Target Selection Button
 */
class BuildTargetSelectionButton extends StatusBarButton {
    private static readonly _noBuildTargetSelected = 'No Build Target Selected';

    settingsName = 'buildTarget';
    constructor(protected readonly config: CargoConfigurationReader, protected readonly priority: number) {
        super(config, priority);
        this.hidden = false;
        this.command = 'cargo-tools.selectBuildTarget';
        this.icon = 'tools';
        this.tooltip = 'Click to change the active build target';
    }

    protected getTextNormal(): string {
        const text = this.text;
        if (text.length === 0) {
            return BuildTargetSelectionButton._noBuildTargetSelected;
        }
        return this.bracketText;
    }

    protected getTextShort(): string {
        const text = this.getTextNormal();
        if (text.length > 20) {
            return `${text.substr(0, 17)}...]`;
        }
        return text;
    }

    protected getTooltipShort(): string | null {
        if (this.getTextNormal() === this.getTextShort()) {
            return this.prependCargo(this.getTooltipNormal());
        }
        return super.getTooltipShort();
    }
}

/**
 * Run Target Selection Button
 */
class RunTargetSelectionButton extends StatusBarButton {
    private static readonly _noRunTargetSelected = 'No Run Target Selected';

    settingsName = 'runTarget';
    constructor(protected readonly config: CargoConfigurationReader, protected readonly priority: number) {
        super(config, priority);
        this.hidden = false;
        this.command = 'cargo-tools.selectRunTarget';
        this.icon = 'play';
        this.tooltip = 'Click to change the active run target';
    }

    protected getTextNormal(): string {
        const text = this.text;
        if (text.length === 0) {
            return RunTargetSelectionButton._noRunTargetSelected;
        }
        return this.bracketText;
    }

    protected getTextShort(): string {
        const text = this.getTextNormal();
        if (text.length > 20) {
            return `${text.substr(0, 17)}...]`;
        }
        return text;
    }

    protected getTooltipShort(): string | null {
        if (this.getTextNormal() === this.getTextShort()) {
            return this.prependCargo(this.getTooltipNormal());
        }
        return super.getTooltipShort();
    }

    protected isVisible(): boolean {
        // Only show benchmark target when a specific package is selected  
        // Hide when "All" is selected or no package is selected
        // This is controlled by updateTargetButtonsVisibility method
        return super.isVisible();
    }
}

/**
 * Benchmark Target Selection Button
 */
class BenchmarkTargetSelectionButton extends StatusBarButton {
    private static readonly _noBenchmarkTargetSelected = 'No Benchmark Target Selected';

    settingsName = 'benchmarkTarget';
    constructor(protected readonly config: CargoConfigurationReader, protected readonly priority: number) {
        super(config, priority);
        this.hidden = false;
        this.command = 'cargo-tools.selectBenchmarkTarget';
        this.icon = 'zap';
        this.tooltip = 'Click to change the active benchmark target';
    }

    protected getTextNormal(): string {
        const text = this.text;
        if (text.length === 0) {
            return BenchmarkTargetSelectionButton._noBenchmarkTargetSelected;
        }
        return this.bracketText;
    }

    protected getTextShort(): string {
        const text = this.getTextNormal();
        if (text.length > 20) {
            return `${text.substr(0, 17)}...]`;
        }
        return text;
    }

    protected getTooltipShort(): string | null {
        if (this.getTextNormal() === this.getTextShort()) {
            return this.prependCargo(this.getTooltipNormal());
        }
        return super.getTooltipShort();
    }
}

/**
 * Feature Selection Button
 */
class FeatureSelectionButton extends StatusBarButton {
    private static readonly _noFeaturesSelected = 'No Features';
    private static readonly _allFeaturesSelected = 'All Features';

    settingsName = 'features';
    constructor(protected readonly config: CargoConfigurationReader, protected readonly priority: number) {
        super(config, priority);
        this.hidden = false;
        this.command = 'cargo-tools.selectFeatures';
        this.icon = 'list-unordered';
        this.tooltip = 'Click to change the active features';
    }

    protected getTextNormal(): string {
        const text = this.text;
        if (text.length === 0) {
            return FeatureSelectionButton._noFeaturesSelected;
        }
        if (text === 'all-features') {
            return FeatureSelectionButton._allFeaturesSelected;
        }
        return this.bracketText;
    }

    protected getTextShort(): string {
        const text = this.getTextNormal();
        if (text.length > 20) {
            return `${text.substr(0, 17)}...]`;
        }
        return text;
    }

    protected getTooltipShort(): string | null {
        if (this.getTextNormal() === this.getTextShort()) {
            return this.prependCargo(this.getTooltipNormal());
        }
        return super.getTooltipShort();
    }
}

/**
 * Build Action Button - executes build action
 */
class BuildActionButton extends StatusBarButton {
    readonly settingsName = 'buildAction';

    constructor(config: CargoConfigurationReader, priority: number) {
        super(config, priority);
        this.button.command = 'cargo-tools.projectStatus.build';
    }

    protected getTextNormal(): string {
        return '$(tools)';
    }

    protected getTooltipNormal(): string {
        return 'Build current target';
    }
}

/**
 * Run Action Button - executes run action
 */
class RunActionButton extends StatusBarButton {
    readonly settingsName = 'runAction';

    constructor(config: CargoConfigurationReader, priority: number) {
        super(config, priority);
        this.button.command = 'cargo-tools.projectStatus.run';
    }

    protected getTextNormal(): string {
        return '$(play)';
    }

    protected getTooltipNormal(): string {
        return 'Run current target';
    }
}

/**
 * Test Action Button - executes test action
 */
class TestActionButton extends StatusBarButton {
    readonly settingsName = 'testAction';

    constructor(config: CargoConfigurationReader, priority: number) {
        super(config, priority);
        this.button.command = 'cargo-tools.projectStatus.test';
    }

    protected getTextNormal(): string {
        return '$(beaker)';
    }

    protected getTooltipNormal(): string {
        return 'Test current package';
    }
}

/**
 * Benchmark Action Button - executes benchmark action
 */
class BenchmarkActionButton extends StatusBarButton {
    readonly settingsName = 'benchmarkAction';

    constructor(config: CargoConfigurationReader, priority: number) {
        super(config, priority);
        this.button.command = 'cargo-tools.projectStatus.bench';
    }

    protected getTextNormal(): string {
        return '$(dashboard)';
    }

    protected getTooltipNormal(): string {
        return 'Benchmark current target';
    }
}

/**
 * Main Status Bar Provider following CMake Tools pattern
 */
export class StatusBarProvider implements vscode.Disposable {
    // Status bar buttons in priority order (higher priority = left-most)
    private readonly _profileButton: ProfileSelectionButton;
    private readonly _packageButton: PackageSelectionButton;
    private readonly _buildTargetButton: BuildTargetSelectionButton;
    private readonly _buildActionButton: BuildActionButton;
    private readonly _runTargetButton: RunTargetSelectionButton;
    private readonly _runActionButton: RunActionButton;
    private readonly _testActionButton: TestActionButton;
    private readonly _benchmarkTargetButton: BenchmarkTargetSelectionButton;
    private readonly _benchmarkActionButton: BenchmarkActionButton;
    private readonly _featuresButton: FeatureSelectionButton;

    private readonly _buttons: StatusBarButton[];

    constructor(private readonly _config: CargoConfigurationReader) {
        // Initialize buttons after config is set
        // Priorities: selection button, then action button next to it (slightly lower priority)
        this._profileButton = new ProfileSelectionButton(this._config, 4.0);
        this._packageButton = new PackageSelectionButton(this._config, 3.9);
        this._testActionButton = new TestActionButton(this._config, 3.85);
        this._buildTargetButton = new BuildTargetSelectionButton(this._config, 3.8);
        this._buildActionButton = new BuildActionButton(this._config, 3.75);
        this._runTargetButton = new RunTargetSelectionButton(this._config, 3.7);
        this._runActionButton = new RunActionButton(this._config, 3.65);
        this._benchmarkTargetButton = new BenchmarkTargetSelectionButton(this._config, 3.6);
        this._benchmarkActionButton = new BenchmarkActionButton(this._config, 3.55);
        this._featuresButton = new FeatureSelectionButton(this._config, 3.5);

        this._buttons = [
            this._profileButton,
            this._packageButton,
            this._testActionButton,
            this._buildTargetButton,
            this._buildActionButton,
            this._runTargetButton,
            this._runActionButton,
            this._benchmarkTargetButton,
            this._benchmarkActionButton,
            this._featuresButton
        ];

        // Subscribe to configuration changes
        this._config.onChange('statusBar', () => this.update());
        this.update();
    }

    dispose(): void {
        this._buttons.forEach(btn => btn.dispose());
    }

    update(): void {
        this._buttons.forEach(btn => btn.update());
    }

    setVisible(v: boolean): void {
        this._buttons.forEach(btn => btn.forceHidden = !v);
    }

    // Profile methods
    setProfileName(profile: CargoProfile): void {
        this._profileButton.text = profile;
    }

    // Package methods
    setPackageName(packageName: string | undefined): void {
        this._packageButton.text = packageName || 'All';
    }

    // Target methods
    setBuildTargetName(targetName: string | null): void {
        this._buildTargetButton.text = targetName || '';
    }

    setRunTargetName(targetName: string | null): void {
        this._runTargetButton.text = targetName || '';
    }

    setBenchmarkTargetName(targetName: string | null): void {
        this._benchmarkTargetButton.text = targetName || '';
    }

    // Feature methods
    setFeaturesText(featuresText: string): void {
        this._featuresButton.text = featuresText;
    }

    // Update buttons based on package selection
    updateTargetButtonsVisibility(packageSelected: boolean): void {
        // Run target and benchmark target are only available when a specific package is selected
        this._runTargetButton.hidden = !packageSelected;
        this._runActionButton.hidden = !packageSelected;
        this._benchmarkTargetButton.hidden = !packageSelected;
        this._benchmarkActionButton.hidden = !packageSelected;
    }
}
