import Cloneable from "../base/cloneable";
import Equatable from "../base/equals";
import Serializable from "../base/serializable";

export class Policy implements Cloneable<Policy>, Equatable<Policy>, Serializable {
    public constructor(
        public name: string,
        public is_subscription: boolean,
        public option?: string,
    ) {
    }

    public static deserialize(policy: Policy) {
        return new Policy(
            policy.name,
            policy.is_subscription,
            policy.option,
        );
    }

    public clone(): Policy {
        return new Policy(this.name, this.is_subscription, this.option);
    }

    public equals(other?: Policy): boolean {
        if (!other) return false;
        return this.name === other.name
            && this.is_subscription === other.is_subscription
            && this.option === other.option;
    }

    public serialize(): any {
        return {
            name: this.name,
            is_subscription: this.is_subscription,
            option: this.option,
        };
    }
}
