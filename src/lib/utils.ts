import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const mapToObject = (map: Map<any, any>) => {
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	const result: Record<string, any> = {};

	for (const key of map.keys()) {
		const value = map.get(key);
		if (value instanceof Map) {
			result[key] = mapToObject(value);
		} else {
			result[key] = value;
		}
	}

	return result;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const jsonParseMap = (map: any) =>
	JSON.stringify(mapToObject(map));

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChild<T> = T extends { child?: any } ? Omit<T, "child"> : T;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChildren<T> = T extends { children?: any } ? Omit<T, "children"> : T;
export type WithoutChildrenOrChild<T> = WithoutChildren<WithoutChild<T>>;
export type WithElementRef<T, U extends HTMLElement = HTMLElement> = T & { ref?: U | null };
