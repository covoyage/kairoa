import { redirect } from '@sveltejs/kit';
import type { PageLoad } from './$types';

export const load: PageLoad = async () => {
  throw redirect(302, '/encode-decode?type=jwt');
};

