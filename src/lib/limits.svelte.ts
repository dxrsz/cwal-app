let limitsStore = $state({ numApiRequests: 0, numReplayDownloads: 0 });

export const getLimitsStore = () => limitsStore;
