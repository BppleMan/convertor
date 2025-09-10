export default interface Deserializable<T> {
    deserialize(input: T): T;
}
