<script lang="ts">
export default {
  props: ['borderclass', 'shadowed', 'isWall', 'x', 'y']
}
</script>

<template>
  <div class="hex">
    <div :class="borderclass"></div>
    <div :class="'filter' + (shadowed ? ' shadowed' : '')"></div>
    <div :class="'hex-inner' + (isWall ? ' is-wall' : '')"> <!-- </div>:style="'background-position: left -' + (68 * x + ((y % 2 == 0) ? 0 : 34)) + 'px
      top -' + 60 * y + 'px;'" -->
    </div>
  </div>
</template>

<style scoped>
.hex {
  width: 68px;
  height: 74px;
  display: inline-block;
  position: relative;
}

.border {
  width: 68px;
  height: 78px;
  -webkit-clip-path: polygon(50% 0%, 100% 25%, 100% 75%, 50% 100%, 0% 75%, 0% 25%);
  clip-path: polygon(50% 0%, 100% 25%, 100% 75%, 50% 100%, 0% 75%, 0% 25%);
  position: absolute;
  background-color: rgba(220, 220, 220, 0.4);
}


.hex-inner.is-wall {
  background: rgb(143, 69, 46);
}

.filter.shadowed {
  background-color: rgba(0, 0, 0, 0.4);
}

.filter {
  width: 68px;
  height: 78px;
  -webkit-clip-path: polygon(50% 0%, 100% 25%, 100% 75%, 50% 100%, 0% 75%, 0% 25%);
  clip-path: polygon(50% 0%, 100% 25%, 100% 75%, 50% 100%, 0% 75%, 0% 25%);
  position: absolute;
  z-index: 10;
}

.glow {
  background: white;
}

.red-glow {
  background: red;
}

.hex-inner {
  width: 64px;
  height: 74px;
  background: #8a8
    /*url('../assets/background.png')*/
  ;
  background-repeat: no-repeat;
  background-size: initial;
  -webkit-clip-path: polygon(50% 0%, 100% 25%, 100% 75%, 50% 100%, 0% 75%, 0% 25%);
  clip-path: polygon(50% 0%, 100% 25%, 100% 75%, 50% 100%, 0% 75%, 0% 25%);
  position: absolute;
  left: 2px;
  top: 2px;

}
</style>
