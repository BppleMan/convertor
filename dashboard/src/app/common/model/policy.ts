export class Policy {
    public constructor(
        public name: string,
        public option: string | null,
        public is_subscription: boolean,
    ) {
    }

    public static deserialize(policy: Policy) {
        return new Policy(
            policy.name,
            policy.option,
            policy.is_subscription,
        );
    }
}
