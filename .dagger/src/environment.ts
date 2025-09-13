export enum Profile {
    Development = "development",
    Production = "production",
}

export class ProfileConfig {
    public constructor(
        public profile: Profile,
    ) {
    }

    public ng_configuration(): string {
        switch (this.profile) {
            case Profile.Development:
                return "development";
            case Profile.Production:
                return "production";
        }
    }

    public cargo_profile(): string {
        switch (this.profile) {
            case Profile.Development:
                return "dev";
            case Profile.Production:
                return "alpine";
        }
    }

    public cargo_target_dir(): string {
        switch (this.profile) {
            case Profile.Development:
                return "debug";
            case Profile.Production:
                return "alpine";
        }
    }

    public imageLabel(version: string): string {
        switch (this.profile) {
            case Profile.Development:
                return "local/convd:dev";
            case Profile.Production:
                return `ghcr.io/convd:${version}`;
        }
    }

    // public like(other: string): boolean {
    //     const name = this.name.toLowerCase();
    //     other = other.toLowerCase();
    //     return name.includes(other);
    // }
    //
    // public static fromString(name: string): Profile | undefined {
    //     return Profile.Profiles.find(profile => profile.like(name));
    // }
}
