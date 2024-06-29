import { defineStore } from 'pinia'


export const useGamesStore = defineStore('games', {
    state: () => {
        let games = []
        let scenarios = []
        return { games, scenarios }
    },
    actions: {
        get_games() {
            fetch("/game").then((r) => r.json().then((v) => { this.games = v }))
        },
        get_scenarios() {
            fetch("/scenario").then((r) => r.json().then((v) => { this.scenarios = v }))
        }
    }
});