import { type ClassValue, clsx } from "clsx";
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
};

export const avatarOrDefault = (avatar?: string) => {
  return (
    avatar ||
    "https://scrassets.classic.blizzard.com/avatar-icons/S1/d2edd0800865e3a5e0c0c86898010f90.png"
  );
};

export function debounce<T extends (...args: any[]) => any>(
  func: T,
  delay: number,
): (...args: Parameters<T>) => void {
  let timeoutId: ReturnType<typeof setTimeout> | null = null;

  return function (this: ThisParameterType<T>, ...args: Parameters<T>): void {
    if (timeoutId) {
      clearTimeout(timeoutId);
    }
    timeoutId = setTimeout(() => {
      func.apply(this, args);
      timeoutId = null; // Clear timeoutId after execution
    }, delay);
  };
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const jsonParseMap = (map: any) => JSON.stringify(mapToObject(map));

// merge tailwind class names
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChild<T> = T extends { child?: any } ? Omit<T, "child"> : T;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChildren<T> = T extends { children?: any }
  ? Omit<T, "children">
  : T;
export type WithoutChildrenOrChild<T> = WithoutChildren<WithoutChild<T>>;
export type WithElementRef<T, U extends HTMLElement = HTMLElement> = T & {
  ref?: U | null;
};
