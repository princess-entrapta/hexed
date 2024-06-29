<script lang="ts">
import { useDeployStore } from './stores/deploy';
import Deployment from './components/Deployment.vue'
import Tile from './components/Tile.vue'
export default {
    props: ["gameId"],
    data() {
        let dps = useDeployStore();
        dps.connect(this.gameId);
        dps.idx = 0
        return { dps }
    },
    components: {
        Tile,
        Deployment,
    },
}

</script>

<template>
    <div class="wrapper">
        <select v-model="dps.idx">
            <option v-for="(dpl, idx) in dps.deployments" :value="idx">Deployment {{ idx + 1 }}</option>
        </select>
        <Deployment v-if="dps.deployments && dps.deployments.length > 0"
            :entities="dps.deployments[dps.idx].allowed_clases" :tiles="dps.deployments[dps.idx].drop_tiles"
            :deployedEntities="dps.deployments[dps.idx].entities">
        </Deployment>
    </div>
</template>

<style scoped>
.wrapper {
    display: flex;
    flex-direction: column;
    align-items: start;
    width: 100vw;
}
</style>
