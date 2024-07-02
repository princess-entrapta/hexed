<script lang="ts">
import { useGridStore } from '@/stores/grid'
import Creature from './Creature.vue'


export default {
    data() {
        let grid = useGridStore()
        return { grid }
    },
    components: {
        Creature
    }
}

</script>



<template>
    <div class="panel">

        <span v-for="log in grid.actionLog">
            <Creature v-if="log.caster != null" :game_class="grid.entities_by_id[log.caster].game_class"
                :hp="grid.entities_by_id[log.caster].resources.hp.current"
                :hp_max="grid.entities_by_id[log.caster].resources.hp.max"
                :team="grid.entities_by_id[log.caster].owner == grid.user ? 'allied' : 'ennemy'"></Creature>
            {{ log.action_name }}
            <Creature v-if="log.target_entity != null" :game_class="grid.entities_by_id[log.target_entity].game_class"
                :hp="grid.entities_by_id[log.target_entity].resources.hp.current"
                :hp_max="grid.entities_by_id[log.target_entity].resources.hp.max"
                :team="grid.entities_by_id[log.target_entity].owner == grid.user ? 'allied' : 'ennemy'"></Creature>
        </span>
    </div>
</template>

<style scoped>
.menu {
    background-color: #333;
    border: 1px solid #999;
    width: 200px;
    text-align: center;
}

.btn {
    cursor: pointer;
}

.panel {
    display: flex;
    flex-direction: column;
}

.panel span {
    max-width: 200px;
    height: 80px;
    display: flex;
    flex-direction: row;
    line-height: 80px;
}

.panel span>* {
    margin: 8px;
}

.options-toggle {
    display: flex;
    flex-direction: row;
    box-sizing: border-box;
}

.selected {
    background-color: #444;
    border: 1px solid #999;
}

textarea {
    width: 100%;
    color: white;
    background: #333;
}

.status-dropdown input {
    width: 70px;
    background-color: #444;
    color: white
}

.spawn-dropdown img {
    display: inline-block;
    width: 64px;
    height: 64px;
}

.delete {
    border: 2px dotted #999;
    height: 100px;
    text-align: center;
    line-height: 100px;
}

.red {
    color: #aa2222;
}

.green {
    color: #22aa22
}
</style>