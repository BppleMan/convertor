export class Profile {
    public static readonly Development: Profile = new Profile("development", "development", "dev");
    public static readonly Production: Profile = new Profile("production", "production", "release");

    public static readonly Profiles: Profile[] = [
        Profile.Development,
        Profile.Production,
    ];

    private constructor(
        public name: string,
        public ng_configuration: string,
        public cargo_profile: string,
    ) {
    }

    public like(other: string): boolean {
        const name = this.name.toLowerCase();
        other = other.toLowerCase();
        return name.includes(other);
    }

    public static fromString(name: string): Profile | undefined {
        return Profile.Profiles.find(profile => profile.like(name));
    }
}
