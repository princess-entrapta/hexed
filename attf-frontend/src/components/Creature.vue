<script lang="ts">

export default {
    props: ['game_class', 'team', 'hp', 'hp_max'],
}
</script>

<template>
    <div class="position">
        <img class="tile-avatar" :src="`/src/assets/${game_class}.png`">
        <span :class="'team ' + team"></span>
        <span :class="'hp_max'"></span>
        <span :class="'hp ' + team" :style="'width: ' + hp / hp_max * 40 + 'px;'"></span>
    </div>
</template>

<style>
.position {
    position: relative;
    width: 60px;
    height: 60px;
}

.hp_max {
    height: 4px;
    width: 40px;
    top: 52px;
    left: 11px;
    background-color: black;
    display: inline-block;
    position: absolute;
    border: 1px solid #bb8;
    z-index: 3;
}

.hp {
    height: 4px;
    top: 53px;
    left: 12px;
    display: inline-block;
    position: absolute;
    z-index: 4;
}

.tile-avatar {
    height: 60px;
    width: 60px;
    position: absolute;
}

.team {
    width: 8px;
    border-radius: 4px;
    height: 8px;
    top: 50px;
    left: 3px;
    display: inline-block;
    position: absolute;
    z-index: 6;
    border: 1px solid #ff8;
}

.ennemy {
    background-color: #b22;
}

.team.playing {
    background-color: #ffc;
}

.hp.playing,
.allied {
    background-color: #5a2;
}
</style>