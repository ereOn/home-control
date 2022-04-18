import { writable, readable, derived, get } from "svelte/store";

async function apiGetStatus() {
	return await (await fetch('/api/v1/status')).json();
}

const initialState = {
	status: {},
	isLoading: false,
	error: ""
};

function createApiStore() {
	const { subscribe, update, set } = writable(initialState);

	const api = {
		subscribe,
		init: async () => {
			update(state => (state = { ...state, isLoading: true }));

			try {
				const status = await (await fetch('/api/v1/status')).json();
				update(state => (state = { ...state, status: status }));
			} catch (e) {
				update(state => (state = { ...state, error: e.message }));
			} finally {
				update(state => (state = { ...state, isLoading: false }));
			}
		},
	};

	let apiStatusPoller;

	function setupApiStatusPoller() {
		if (apiStatusPoller) {
			clearInterval(apiStatusPoller);
		}

		apiStatusPoller = setInterval(api.init, 1000);
	}

	setupApiStatusPoller();

	return api;
}

export const api = createApiStore();