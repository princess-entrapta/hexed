<script lang="ts">
import HelloWorld from './HelloWorld.vue';
import Creature from './Creature.vue';
import { Entity, useGridStore, useMouseInfoStore } from '@/stores/grid';
import { Coords } from '@/stores/grid';

export default {
    props: {
        tileType: { type: String, required: true },
        entity: { type: Entity || null },
        x: { type: Number, required: true },
        y: { type: Number, required: true },
        shadowed: { type: Boolean },
    },
    data() {
        return { isHovered: false, isSelected: false }
    },
    components: {
        HelloWorld,
        Creature,
    },
    computed: {
        isPlaying() {
            return useGridStore().isPlaying;
        },
        isEnemy() {
            let grid = useGridStore()
            return this.entity && this.entity.owner != grid.user
        },
        isCurrent() {
            let grid = useGridStore()
            return this.entity && this.entity.id == grid.playing
        },
        isValidTarget() {
            let grid = useGridStore()
            if (!grid.ability) {
                return true
            }
            for (let t in grid.ability.targets) {
                if (grid.ability.targets[t].x == this.x && grid.ability.targets[t].y == this.y)
                    return true
            }
            return false
        },
    },
    emits: ["targetSelect"],
    methods: {
        mouseMove(ev: MouseEvent) {
            const raw_y = ev.clientY - this.$el.getBoundingClientRect().y
            const raw_x = ev.clientX - this.$el.getBoundingClientRect().x
            const cur_dist = (raw_x - 34) * (raw_x - 34) / (34 * 34) + (raw_y - 37) * (raw_y - 37) / (37 * 37)

            var mouseStore = useMouseInfoStore()
            if (!this.isHovered && cur_dist < 1) {
                mouseStore.onHighlight()
                this.isHovered = this.isEnemy ? 'red-glow' : 'glow'
                mouseStore.onHighlight = () => {
                    this.isHovered = ''
                }
            }
            else if (this.isHovered && cur_dist > 1) {
                this.isHovered = this.isEnemy ? 'red-glow' : 'glow'
            }
            else if (mouseStore.tileSelected && mouseStore.tileSelected.x == this.x && mouseStore.tileSelected.y == this.y) {
                this.isSelected = this.isEnemy ? 'red-glow' : 'glow'
            }
        },
        mouseLeave(ev: MouseEvent) {
            this.isHovered = ''
        },
        onClick(ev: MouseEvent) {
            if (!this.isValidTarget) {
                return
            }
            var mouseStore = useMouseInfoStore()
            mouseStore.tileSelected = new Coords(this.x, this.y)
            let grid = useGridStore()
            if (grid.proposedAbilities.length == 1) {
                grid.ability = grid.abilities[grid.proposedAbilities[0]];
            }
            if (grid.ability && grid.gameId !== null && grid.gameId !== undefined) {
                grid.isPlaying = false
                grid.useAbility(this.x, this.y)
            }
        },
        drop() {
            var mouseInfo = useMouseInfoStore()

        },
    }
}

</script>


<template>
    <div class="tile-wrapper" v-on:dragover="dragOver" v-on:drop="drop()" @click.stop="onClick" @mouseover="mouseMove"
        :style="'left: ' + (parseInt(x) * 36 + 200) + 'px;top: ' + (parseInt(y) * 62 + 200) + 'px; '">
        <div class="avatar tile">
            <Creature v-if="entity" :game_class="entity.game_class"
                :hp="entity.resources ? entity.resources.hp.current : 100"
                :hp_max="entity.resources ? entity.resources.hp.max : 100"
                :team="isCurrent ? 'playing' : (isEnemy ? 'ennemy' : 'allied')" />
        </div>
        <div class="tile">
            <HelloWorld :shadowed="shadowed || !isPlaying || !isValidTarget" :isWall="tileType == 'Wall'"
                :borderclass="'border ' + isHovered" :x="x" :y="y" />
        </div>
    </div>
</template>

<style>
.avatar {
    z-index: 2;
    left: 4px;
    top: 8px;
}

.tile {
    display: inline-block;
    position: absolute;
}

.tile-wrapper {
    position: relative;
    display: inline-block;
    z-index: 1;
}
</style>