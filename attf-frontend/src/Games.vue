<script lang="ts">
import { useGamesStore } from './stores/games';
export default {
    data() {
        let gms = useGamesStore();
        gms.get_games();
        gms.get_scenarios();
        return { gms: gms }
    },
    methods: {
        createScenario(scenarioId) {
            fetch('/game', { method: 'POST', headers: { 'Content-type': 'application/json' }, body: JSON.stringify(scenarioId) })

        }
    },
    components: {
    },
}

</script>

<template>
    <div>
        <div v-for="game in gms.games"><a v-if="game.game_status === 'Running'" :href="'/play/' + game.id">Resume game
                {{
            game.id }}</a>
            <a v-else :href="'/create/' + game.id">Seat in new game {{
            game.id }} scenario "{{
            game.scenario_description }}"</a>
        </div>
    </div>
    <div>
        <button v-for="scenario in gms.scenarios" @click="createScenario(scenario)">Create scenario {{ scenario
            }}</button>
    </div>
</template>

<style scoped></style>
