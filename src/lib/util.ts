// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const jsonParseMap = (map: any) =>
	JSON.stringify(map, (key, value) => {
		if (value instanceof Map) {
			return {
				dataType: 'Map',
				value: Array.from(value.entries()) // or with spread: value: [...value]
			};
		} else {
			return value;
		}
	});
