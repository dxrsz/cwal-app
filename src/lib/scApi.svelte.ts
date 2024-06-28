import { BroodWarConnection, SCApi } from 'bw-web-api';
import { scrState } from './scrState.svelte';
import { GravaticBooster, SCApiWithCaching } from 'gravatic-booster';

const createGB = async (port: number | null): Promise<GravaticBooster | null> =>
    port != null ? await GravaticBooster.create(
        new SCApiWithCaching(
            new SCApi(
            new BroodWarConnection(`http://localhost:${port}`)))) : null;

const gb = $derived(createGB(scrState.port));

/**
 * Call this to get the SCApi object, which is used to interact with the SC:R api.
 * 
 * @returns The SCApi object, or null if the port is not open.
 */
export const getGb = () => gb;